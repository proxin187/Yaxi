
// page: 148


#[non_exhaustive]
pub struct Protocol;

impl Protocol {
    pub const ERROR: u8 = 0;
    pub const REPLY: u8 = 1;

    pub const KEY_PRESS: u8 = 2;
    pub const KEY_RELEASE: u8 = 3;
}

pub enum EventKind {
}


