
// page: 148


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
}

pub enum EventKind {
}


