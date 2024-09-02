use std::fs::File;
use std::io::Read;
use std::env;


#[derive(Debug)]
pub struct Entry {
    name: Vec<u8>,
    data: Vec<u8>,
}

pub struct XAuth {
    file: File,
}

impl XAuth {
    pub fn new() -> Result<XAuth, Box<dyn std::error::Error>> {
        Ok(XAuth {
            file: File::open(env::var("XAUTHORITY")?)?,
        })
    }

    fn read(&mut self, size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf: Vec<u8> = vec![0u8; size];

        self.file.read_exact(&mut buf)?;

        Ok(buf)
    }

    fn value(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let size = self.read(2)?;

        self.read(((size[0] as u16) << 8 | size[1] as u16) as usize)
    }

    pub fn entry(&mut self) -> Result<Entry, Box<dyn std::error::Error>> {
        // TODO: finish up whatever tf this is

        Ok(Entry {
            name: self.value()?,
            data: self.value()?,
        })
    }
}


