mod request;
mod error;

use error::Error;
use request::*;

use std::os::unix::net::UnixStream;
use std::net::{SocketAddr, TcpStream};
use std::io::{Read, Write};

// https://www.x.org/docs/XProtocol/proto.pdf

const X_TCP_PORT: u16 = 6000;
const X_PROTOCOL: u16 = 11;
const X_PROTOCOL_REVISION: u16 = 0;

pub struct Display<T> {
    stream: Box<T>,
    events: Vec<Event>,
}

impl<T> Display<T> where T: Read + Write {
    pub fn connect(stream: T) -> Result<Display<T>, Box<dyn std::error::Error>> {
        let mut display = Display {
            stream: Box::new(stream),
            events: Vec::new(),
        };

        display.setup()?;

        Ok(display)
    }

    fn recv(&mut self, size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; size];

        match self.stream.read_exact(&mut buffer) {
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

    fn endian(&self) -> u8 {
        cfg!(target_endian = "little")
            .then(|| 0x6c)
            .unwrap_or_else(|| 0x42)
    }

    fn read_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response: SuccessResponse = self.recv_decode()?;

        println!("response: {:?}", response);

        let vendor = self.recv_str(response.vendor_len as usize)?;

        println!("vendor: {}", vendor);

        let bytes = self.recv(std::mem::size_of::<PixmapFormat>() * response.pixmap_formats_len as usize)?;

        let formats: &[PixmapFormat] = request::decode_slice(&bytes, response.pixmap_formats_len as usize);

        println!("formats: {:?}", formats);

        for _ in 0..response.roots_len {
            let screen: Screen = self.recv_decode()?;

            for _ in 0..screen.allowed_depths_len {
                let depth: Depth = self.recv_decode()?;

                let bytes = self.recv(std::mem::size_of::<Visual>() * depth.visuals_len as usize)?;

                let visuals: &[Visual] = request::decode_slice(&bytes, depth.visuals_len as usize);

                println!("visuals: {:?}", visuals);
            }
        }

        Ok(())
    }

    fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = SetupRequest::new(self.endian(), X_PROTOCOL, X_PROTOCOL_REVISION);

        self.stream.write_all(request::encode(&request))?;

        let bytes = self.recv(8)?;
        let response: SetupResponse = request::decode(&bytes);

        // TODO: setup response is only used as a temporary type before we know what type it really
        // has

        println!("response: {:?}", response);

        match response.status {
            1 => self.read_setup(),
            0 => Err(Box::new(Error::SetupFailed)),
            2 => Err(Box::new(Error::Authenthicate)),
            _ => Err(Box::new(Error::InvalidStatus)),
        }
    }

    pub fn poll_events(&mut self) {
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


