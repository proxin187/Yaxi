pub mod window;
pub mod error;
pub mod proto;
mod request;
mod auth;
mod xid;

use error::Error;
use proto::*;
use request::*;
use window::*;

use std::os::unix::net::UnixStream;
use std::net::{SocketAddr, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::thread;

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock mutex"))
    }
}

// https://www.x.org/docs/XProtocol/proto.pdf

const X_TCP_PORT: u16 = 6000;
const X_PROTOCOL: u16 = 11;
const X_PROTOCOL_REVISION: u16 = 0;

pub trait TryClone {
    fn try_clone(&self) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}

impl TryClone for File {
    fn try_clone(&self) -> Result<Box<File>, Box<dyn std::error::Error>> {
        self.try_clone()
            .map(|stream| Box::new(stream))
            .map_err(|err| err.into())
    }
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

    fn send(&mut self, request: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.write_all(request).map_err(|err| err.into())
    }

    fn send_arr(&mut self, requests: &[Vec<u8>]) -> Result<(), Box<dyn std::error::Error>> {
        for request in requests {
            self.send(request)?;
        }

        Ok(())
    }

    fn send_pad(&mut self, request: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.send(request)?;

        self.send(&request::pad(request.len()))?;

        Ok(())
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

pub struct Atom {
    id: u32,
}

#[derive(Debug, Clone)]
pub struct Visual {
    pub id: u32,
    pub class: VisualClass,
}

impl Visual {
    pub fn new(response: VisualResponse) -> Visual {
        Visual {
            id: response.visual_id,
            class: VisualClass::from(response.class),
        }
    }
}

pub struct Depth {
    depth: u8,
    length: u16,
    visuals: Vec<Visual>,
}

impl Depth {
    pub fn new(response: DepthResponse) -> Depth {
        Depth {
            depth: response.depth,
            length: response.visuals_len,
            visuals: Vec::new(),
        }
    }

    pub fn extend(&mut self, responses: &[VisualResponse]) {
        self.visuals.extend(responses.iter().map(|response| Visual::new(*response)));
    }
}

pub struct Screen {
    response: ScreenResponse,
    depths: Vec<Depth>,
}

impl Screen {
    pub fn new(response: ScreenResponse) -> Screen {
        Screen {
            response,
            depths: Vec::new(),
        }
    }
}

pub struct Display<T> {
    stream: Stream<T>,
    events: Arc<Mutex<Vec<Event>>>,
    replies: Arc<Mutex<Vec<Reply>>>,
    roots: Vec<Screen>,
    sequence: SequenceManager,
}

impl<T> Display<T> where T: Send + Sync + Read + Write + TryClone + 'static {
    pub fn connect<'a>(inner: T) -> Result<Display<T>, Box<dyn std::error::Error>> {
        let mut display = Display {
            stream: Stream::new(inner),
            events: Arc::new(Mutex::new(Vec::new())),
            replies: Arc::new(Mutex::new(Vec::new())),
            roots: Vec::new(),
            sequence: SequenceManager::new(),
        };

        display.setup()?;

        Ok(display)
    }

    pub fn poll_reply(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(!lock!(self.replies)?.is_empty())
    }

    pub fn wait_for_reply(&mut self) -> Result<Reply, Box<dyn std::error::Error>> {
        while !self.poll_reply()? {}

        lock!(self.replies)?.pop().ok_or(Box::new(Error::NoReply))
    }

    pub fn default_root_window(&self) -> Result<Window<T>, Box<dyn std::error::Error>> {
        let stream = self.stream.try_clone()?;
        let screen = self.roots.first().ok_or(Error::NoScreens)?;

        Ok(Window::<T>::new(stream, self.sequence.clone(), self.visual_from_id(screen.response.root_visual)?, screen.response.root_depth, screen.response.root))
    }

    pub fn visual_from_id(&self, id: u32) -> Result<Visual, Box<dyn std::error::Error>> {
        for screen in &self.roots {
            for depth in &screen.depths {
                match depth.visuals.iter().find(|visual| visual.id == id) {
                    Some(visual) => return Ok(visual.clone()),
                    None => {},
                }
            }
        }

        Err(Box::new(Error::InvalidId))
    }

    pub fn intern_atom(&mut self, name: &str, only_if_exists: bool) -> Result<(), Box<dyn std::error::Error>> {
        let request = InternAtom {
            opcode: Opcode::INTERN_ATOM,
            only_if_exists: only_if_exists.then(|| 1).unwrap_or(0),
            length: 2 + (name.len() as u16 + request::pad(name.len()).len() as u16) / 4,
            name_len: name.len() as u16,
            pad1: [0u8; 2],
        };

        self.stream.send(request::encode(&request))?;

        self.stream.send_pad(name.as_bytes())?;

        self.sequence.append(ReplyKind::InternAtom)?;

        Ok(())
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

        println!("vendor: {}", vendor);

        let bytes = self.stream.recv(std::mem::size_of::<PixmapFormat>() * response.pixmap_formats_len as usize)?;

        let formats: &[PixmapFormat] = request::decode_slice(&bytes, response.pixmap_formats_len as usize);

        // println!("formats: {:?}", formats);

        for _ in 0..response.roots_len {
            let mut screen = Screen::new(self.stream.recv_decode()?);

            for _ in 0..screen.response.allowed_depths_len {
                let mut depth = Depth::new(self.stream.recv_decode()?);

                let bytes = self.stream.recv(std::mem::size_of::<VisualResponse>() * depth.length as usize)?;

                depth.extend(request::decode_slice(&bytes, depth.length as usize));

                screen.depths.push(depth);
            }

            self.roots.push(screen);
        }

        let stream = self.stream.try_clone()?;
        let events = self.events.clone();
        let replies = self.replies.clone();
        let sequence = self.sequence.clone();

        thread::spawn(move || {
            let mut listener = EventListener::new(stream, events, replies, sequence);

            if let Err(err) = listener.listen() {
                println!("[ERROR] listener failed: {}", err);
            }
        });

        xid::setup(response.resource_id_base, response.resource_id_mask)?;

        Ok(())
    }

    fn setup<'a>(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let entry = auth::entry()?;

        let request = SetupRequest::new(self.endian(), X_PROTOCOL, X_PROTOCOL_REVISION, entry.name.len() as u16, entry.data.len() as u16);

        self.stream.send(request::encode(&request))?;

        self.stream.send_arr(&[entry.name.clone(), request::pad(entry.name.len()), entry.data.clone(), request::pad(entry.data.len())])?;

        let response: SetupResponse = self.stream.recv_decode()?;

        match response.status {
            1 => self.read_setup(),
            0 => Err(Box::new(Error::SetupFailed { reason: self.stream.recv_str(response.padding as usize)? })),
            2 => Err(Box::new(Error::Authenthicate)),
            _ => Err(Box::new(Error::InvalidStatus)),
        }
    }
}

pub struct EventListener<T> {
    stream: Stream<T>,
    events: Arc<Mutex<Vec<Event>>>,
    replies: Arc<Mutex<Vec<Reply>>>,
    sequence: SequenceManager,
}

impl<T> EventListener<T> where T: Send + Sync + Read + Write + TryClone {
    pub fn new(stream: Stream<T>, events: Arc<Mutex<Vec<Event>>>, replies: Arc<Mutex<Vec<Reply>>>, sequence: SequenceManager) -> EventListener<T> {
        EventListener {
            stream,
            events,
            replies,
            sequence,
        }
    }

    fn handle_reply(&mut self, event: GenericEvent) -> Result<(), Box<dyn std::error::Error>> {
        let sequence = self.sequence.get(event.sequence)?;

        match sequence.kind {
            ReplyKind::InternAtom => {
                let response: InternAtomResponse = self.stream.recv_decode()?;

                lock!(self.replies)?.push(Reply::InternAtom {
                    atom: response.atom,
                });
            },
        }

        Ok(())
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
                self.handle_reply(event)?;

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

pub fn open_tcp<'a>(display: u16) -> Result<Display<TcpStream>, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], X_TCP_PORT + display)))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}

pub fn open_unix<'a>(display: u16) -> Result<Display<UnixStream>, Box<dyn std::error::Error>> {
    let stream = UnixStream::connect(format!("/tmp/.X11-unix/X{}", display))?;

    stream.set_nonblocking(false)?;

    Ok(Display::connect(stream)?)
}


