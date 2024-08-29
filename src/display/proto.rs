

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

pub enum EventKind {
}


