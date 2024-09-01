use std::sync::atomic::{Ordering, AtomicU16};
use std::sync::{Arc, Mutex};

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to lock mutex"))
    }
}

#[non_exhaustive]
pub struct Response;

impl Response {
    pub const ERROR: u8 = 0;
    pub const REPLY: u8 = 1;

    pub const KEY_PRESS: u8 = 2;
    pub const KEY_RELEASE: u8 = 3;
}

#[non_exhaustive]
pub struct Opcode;

impl Opcode {
    pub const CREATE_WINDOW: u8 = 1;
    pub const DESTROY_WINDOW: u8 = 4;
    pub const MAP_WINDOW: u8 = 8;
}

#[non_exhaustive]
pub struct ErrorCode;

impl ErrorCode {
    pub const REQUEST: u8 = 1;
    pub const VALUE: u8 = 2;
    pub const WINDOW: u8 = 3;
    pub const PIXMAP: u8 = 4;
    pub const ATOM: u8 = 5;
    pub const CURSOR: u8 = 6;
    pub const FONT: u8 = 7;
    pub const MATCH: u8 = 8;
    pub const DRAWABLE: u8 = 9;
    pub const ACCESS: u8 = 10;
    pub const ALLOC: u8 = 11;
    pub const COLORMAP: u8 = 12;
    pub const G_CONTEXT: u8 = 13;
    pub const ID_CHOICE: u8 = 14;
    pub const NAME: u8 = 15;
    pub const LENGTH: u8 = 16;
    pub const IMPLEMENTATION: u8 = 17;
}

pub enum ReplyKind {
    InternAtom,
}

pub struct Sequence {
    id: u16,
    kind: ReplyKind,
}

impl Sequence {
    pub fn new(id: u16, kind: ReplyKind) -> Sequence {
        Sequence {
            id,
            kind,
        }
    }
}

#[derive(Clone)]
pub struct SequenceManager {
    id: Arc<AtomicU16>,
    sequences: Arc<Mutex<Vec<Sequence>>>,
}

impl SequenceManager {
    pub fn new() -> SequenceManager {
        SequenceManager {
            id: Arc::new(AtomicU16::default()),
            sequences: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn next(&mut self, kind: Option<ReplyKind>) -> Result<(), Box<dyn std::error::Error>> {
        let mut lock = lock!(self.sequences)?;

        if let Some(kind) = kind {
            lock.push(Sequence::new(self.id.load(Ordering::Relaxed), kind));
        }

        self.id.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}

pub enum EventKind {
}


