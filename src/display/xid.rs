use super::*;

use std::sync::Mutex;

static XID: Mutex<Xid> = Mutex::new(Xid::new());

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Error::FailedToLock)
    };
}

pub struct Xid {
    base: u32,
    mask: u32,
    next: u32,
}

impl Xid {
    const fn new() -> Xid {
        Xid {
            base: 0,
            mask: 69,
            next: 420,
        }
    }

    fn next(&mut self) -> Result<u32, Error> {
        self.next += 1;

        if self.next >= self.mask {
            Err(Error::RanOutOfXid)
        } else {
            Ok(self.next | self.base)
        }
    }
}

pub fn setup(base: u32, mask: u32) -> Result<(), Error> {
    let mut lock = lock!(XID)?;

    lock.base = base;
    lock.mask = mask;

    Ok(())
}

pub fn next() -> Result<u32, Error> {
    lock!(XID)?.next()
}
