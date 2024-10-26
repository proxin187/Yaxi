use crate::display::request::*;
use crate::display::*;
use crate::proto::*;

use std::io::{Read, Write};


#[non_exhaustive]
pub struct MinorOpcode;

impl MinorOpcode {
    pub const IS_ACTIVE: u8 = 4;
    pub const QUERY_SCREENS: u8 = 5;
}

pub struct Xinerama<T: Send + Sync + Read + Write + TryClone> {
    stream: Stream<T>,
    replies: Queue<Reply>,
    sequence: SequenceManager,
    major_opcode: u8,
}

impl<T> Xinerama<T> where T: Send + Sync + Read + Write + TryClone {
    pub(crate) fn new(stream: Stream<T>, replies: Queue<Reply>, sequence: SequenceManager, major_opcode: u8) -> Xinerama<T> {
        Xinerama {
            stream,
            replies,
            sequence,
            major_opcode,
        }
    }

    /// query the screens and return them in a vector
    pub fn query_screens(&mut self) -> Result<Vec<XineramaScreenInfo>, Error> {
        self.sequence.append(ReplyKind::XineramaQueryScreens)?;

        self.stream.send_encode(XineramaQueryScreens {
            opcode: self.major_opcode,
            minor: MinorOpcode::QUERY_SCREENS,
            length: 1,
        })?;

        match self.replies.wait()? {
            Reply::XineramaQueryScreens { screens } => Ok(screens),
            _ => unreachable!(),
        }
    }

    /// returns true if xinerama is active
    pub fn is_active(&mut self) -> Result<bool, Error> {
        self.sequence.append(ReplyKind::XineramaIsActive)?;

        self.stream.send_encode(XineramaIsActive {
            opcode: self.major_opcode,
            minor: MinorOpcode::IS_ACTIVE,
            length: 1,
        })?;

        match self.replies.wait()? {
            Reply::XineramaIsActive(response) => Ok(response.state != 0),
            _ => unreachable!(),
        }
    }
}


