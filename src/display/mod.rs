pub mod window;
pub mod error;
mod request;
mod proto;
mod xid;

use error::Error;
use proto::*;
use request::*;
use window::*;

use std::os::unix::net::UnixStream;
use std::net::{SocketAddr, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

// https://www.x.org/docs/XProtocol/proto.pdf

const X_TCP_PORT: u16 = 6000;
const X_PROTOCOL: u16 = 11;
const X_PROTOCOL_REVISION: u16 = 0;

static SEQUENCE: u16 = 0;


pub trait TryClone {
    fn try_clone(&self) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}

impl TryClone for TcpStream {
    fn try_clone(&self) -> Result<Box<TcpStream>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
}

impl TryClone for UnixStream {
    fn try_clone(&self) -> Result<Box<UnixStream>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
}

pub struct Stream<T> {
    inner: Box<T>,
}

impl<T> Stream<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(inner: T) -> Stream<T> {
        Stream {
            inner: Box::new(inner),
        }
    }

    fn try_clone(&self) -> Result<Stream<T>, Box<dyn std::error::Error>> {
        Ok(Stream {
            inner: self.inner.try_clone()?,
        })
    }

    fn recv(&mut self, size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; size];

        match self.inner.read_exact(&mut buffer) {
            Ok(()) => Ok(buffer),
            Err(err) => Err(err.into()),
        }
    }

    fn recv_str(&mut self, size: usize) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.recv(size)?;

        self.recv(size % 4)?;

        Ok(String::from_utf8(bytes)?)
    }

    fn recv_decode<R>(&mut self) -> Result<R, Box<dyn std::error::Error>> {
        let bytes = self.recv(std::mem::size_of::<R>())?;

        Ok(request::decode(&bytes))
    }
}


pub struct Display<T> {
    stream: Stream<T>,
    events: Arc<Mutex<Vec<EventKind>>>,
    roots: Vec<Screen>,
}

impl<T> Display<T> where T: Send + Sync + Read + Write + TryClone + 'static {
    pub fn connect(inner: T) -> Result<Display<T>, Box<dyn std::error::Error>> {
        let mut display = Display {
            stream: Stream::new(inner),
            events: Arc::new(Mutex::new(Vec::new())),
            roots: Vec::new(),
        };

        display.setup()?;

        Ok(display)
    }

    pub fn default_root_window(&self) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let stream = self.stream.try_clone()?;
        let screen = self.roots.first().ok_or(Error::NoScreens)?;

        Ok(Window::<T>::new(stream, VisualClass::from(screen.root_visual), screen.root_depth, screen.root))
    }

    fn endian(&self) -> u8 {
        cfg!(target_endian = "little")
            .then(|| 0x6c)
            .unwrap_or_else(|| 0x42)
    }

    fn read_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response: SuccessResponse = self.stream.recv_decode()?;

        println!("response: {:?}", response);

        let vendor = self.stream.recv_str(response.vendor_len as usize)?;

        // println!("vendor: {}", vendor);

        let bytes = self.stream.recv(std::mem::size_of::<PixmapFormat>() * response.pixmap_formats_len as usize)?;

        let formats: &[PixmapFormat] = request::decode_slice(&bytes, response.pixmap_formats_len as usize);

        // println!("formats: {:?}", formats);

        for _ in 0..response.roots_len {
            let screen: Screen = self.stream.recv_decode()?;

            for _ in 0..screen.allowed_depths_len {
                let depth: Depth = self.stream.recv_decode()?;

                let bytes = self.stream.recv(std::mem::size_of::<Visual>() * depth.visuals_len as usize)?;

                let visuals: &[Visual] = request::decode_slice(&bytes, depth.visuals_len as usize);

                // println!("visuals: {:?}", visuals);
            }

            self.roots.push(screen);
        }

        let stream = self.stream.try_clone()?;
        let events = self.events.clone();

        thread::spawn(move || {
            let mut listener = EventListener::new(stream, events);

            if let Err(err) = listener.listen() {
                println!("[ERROR] listener failed: {}", err);
            }
        });

        xid::setup(response.resource_id_base, response.resource_id_mask)?;

        Ok(())
    }

    fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = SetupRequest::new(self.endian(), X_PROTOCOL, X_PROTOCOL_REVISION);

        self.stream.inner.write_all(request::encode(&request))?;

        let bytes = self.stream.recv(8)?;
        let response: SetupResponse = request::decode(&bytes);

        println!("response: {:?}", response);

        match response.status {
            1 => self.read_setup(),
            0 => Err(Box::new(Error::SetupFailed)),
            2 => Err(Box::new(Error::Authenthicate)),
            _ => Err(Box::new(Error::InvalidStatus)),
        }
    }
}

pub struct EventListener<T> {
    stream: Stream<T>,
    events: Arc<Mutex<Vec<EventKind>>>,
}

impl<T> EventListener<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, events: Arc<Mutex<Vec<EventKind>>>) -> EventListener<T> {
        EventListener {
            stream,
            events,
        }
    }

    fn handle_reply(&mut self, event: GenericEvent) {
        match event.sequence {
            _ => {},
        }
    }

    fn handle_event(&mut self, event: GenericEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event.opcode & 0b0111111 {
            Response::ERROR => {
                let error: ErrorEvent = self.stream.recv_decode()?;

                println!("error: {:?}", error);

                Err(Box::new(Error::Event {
                    detail: event.detail,
                    sequence: event.sequence,
                }))
            },
            Response::REPLY => {
                self.handle_reply(event);

                Ok(())
            },
            Response::KEY_PRESS | Response::KEY_RELEASE => {
                let event: KeyEvent = self.stream.recv_decode()?;

                println!("event: {:?}", event);

                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let event: GenericEvent = self.stream.recv_decode()?;

            self.handle_event(event)?;
        }
    }
}

pub fn open_tcp(display: u16) -> Result<Display<TcpStream>, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], X_TCP_PORT + display)))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}

pub fn open_unix(display: u16) -> Result<Display<UnixStream>, Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(format!("/tmp/.X11-unix/X{}", display))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}


