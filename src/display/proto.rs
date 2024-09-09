use super::{Window, WindowClass, Visual, Error, TryClone};

use std::sync::atomic::{Ordering, AtomicU16};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

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
    pub const CHANGE_WINDOW_ATTRIBUTES: u8 = 2;
    pub const GET_WINDOW_ATTRIBUTES: u8 = 3;
    pub const DESTROY_WINDOW: u8 = 4;
    pub const DESTROY_SUBWINDOWS: u8 = 5;
    pub const REPARENT_WINDOW: u8 = 7;
    pub const MAP_WINDOW: u8 = 8;
    pub const MAP_SUBWINDOWS: u8 = 9;
    pub const UNMAP_WINDOW: u8 = 10;
    pub const UNMAP_SUBWINDOWS: u8 = 11;
    pub const INTERN_ATOM: u8 = 16;
    pub const CHANGE_PROPERTY: u8 = 18;
    pub const DELETE_PROPERTY: u8 = 19;
    pub const GET_PROPERTY: u8 = 20;
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

#[derive(Debug, Clone)]
pub enum Reply {
    InternAtom {
        atom: u32
    },
    GetProperty {
        value: Vec<u8>,
    },
    GetWindowAttributes {
        visual: Visual,
        class: WindowClass,
        depth: u8,
    },
}

#[derive(Debug)]
pub enum ReplyKind {
    InternAtom,
    GetProperty,
    GetWindowAttributes,
}

#[derive(Debug)]
pub struct Sequence {
    pub id: u16,
    pub kind: ReplyKind,
}

impl Sequence {
    pub fn new(id: u16, kind: ReplyKind) -> Sequence {
        Sequence {
            id,
            kind,
        }
    }
}

pub struct Queue<T> {
    queue: Arc<Mutex<Vec<T>>>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn clone(&self) -> Queue<T> {
        Queue {
            queue: self.queue.clone(),
        }
    }

    pub fn poll(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(!lock!(self.queue)?.is_empty())
    }

    pub fn wait(&mut self) -> Result<T, Box<dyn std::error::Error>> {
        while !self.poll()? {}

        lock!(self.queue)?.pop().ok_or(Box::new(Error::NoReply))
    }

    pub fn push(&mut self, element: T) -> Result<(), Box<dyn std::error::Error>> {
        lock!(self.queue).map(|mut lock| lock.push(element))
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

    pub fn get(&mut self, id: u16) -> Result<Sequence, Box<dyn std::error::Error>> {
        let mut lock = lock!(self.sequences)?;

        match lock.iter().position(|sequence| sequence.id == id) {
            Some(index) => Ok(lock.remove(index)),
            None => Err(Box::new(Error::InvalidId)),
        }
    }

    pub fn skip(&mut self) {
        self.id.fetch_add(1, Ordering::Relaxed);
    }

    pub fn append(&mut self, kind: ReplyKind) -> Result<(), Box<dyn std::error::Error>> {
        self.skip();

        lock!(self.sequences)?.push(Sequence::new(self.id.load(Ordering::Relaxed), kind));

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum KeyEventKind {
    Press,
    Release,
}

#[derive(Debug, Clone)]
pub struct Coordinates {
    x: u16,
    y: u16,
    root_x: u16,
    root_y: u16,
}

impl Coordinates {
    pub fn new(x: u16, y: u16, root_x: u16, root_y: u16) -> Coordinates {
        Coordinates {
            x,
            y,
            root_x,
            root_y,
        }
    }
}

pub enum Event<T: Send + Sync + Read + Write + TryClone> {
    KeyEvent {
        kind: KeyEventKind,
        coordinates: Coordinates,
        window: Window<T>,
        root: Window<T>,
        subwindow: Window<T>,
        state: u16,
        send_event: bool,
    },
}


