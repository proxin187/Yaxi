use super::{Stream, Error};

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::env;
use std::mem;


#[derive(Debug)]
pub struct Entry {
    family: u16,
    address: Vec<u8>,
    number: Vec<u8>,
    pub name: Vec<u8>,
    pub data: Vec<u8>,
}

pub struct XAuth {
    file: Stream,
}

impl XAuth {
    pub fn new() -> Result<XAuth, Box<dyn std::error::Error>> {
        let file = File::options()
            .read(true)
            .write(true)
            .open(env::var("XAUTHORITY")?)?;

        Ok(XAuth {
            file: Stream::new(Arc::new(Mutex::new(file.try_clone()?)), Arc::new(Mutex::new(file))),
        })
    }

    fn value(&mut self) -> Result<Vec<u8>, Error> {
        let size = self.file.recv(mem::size_of::<u16>())?;

        self.file.recv(((size[0] as u16) << 8 | size[1] as u16) as usize)
    }

    pub fn entry(&mut self) -> Result<Entry, Error> {
        let family = self.file.recv(mem::size_of::<u16>())?;

        Ok(Entry {
            family: (family[0] as u16) << 8 | family[1] as u16,
            address: self.value()?,
            number: self.value()?,
            name: self.value()?,
            data: self.value()?,
        })
    }
}

pub fn entry() -> Result<Entry, Error> {
    let mut auth = XAuth::new().map_err(|_| Error::Stream)?;

    auth.entry()
}


