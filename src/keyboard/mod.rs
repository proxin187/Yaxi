use crate::display::error::Error;

// page 86 @ https://www.x.org/docs/XProtocol/proto.pdf

#[non_exhaustive]
pub struct CharacterSet;

impl CharacterSet {
    pub const LATIN1: u8 = 0;
    pub const LATIN2: u8 = 1;
    pub const LATIN3: u8 = 2;
    pub const LATIN4: u8 = 3;
    pub const KANA: u8 = 4;
    pub const ARABIC: u8 = 5;
    pub const CYRILLIC: u8 = 6;
    pub const GREEK: u8 = 7;
    pub const TECHNICAL: u8 = 8;
    pub const SPECIAL: u8 = 9;
    pub const PUBLISHING: u8 = 10;
    pub const APL: u8 = 11;
    pub const HEBREW: u8 = 12;
    pub const THAI: u8 = 13;
    pub const KOREAN: u8 = 14;
    pub const LATIN5: u8 = 15;
    pub const LATIN6: u8 = 16;
    pub const LATIN7: u8 = 17;
    pub const LATIN8: u8 = 18;
    pub const LATIN9: u8 = 19;
    pub const CURRENCY: u8 = 32;
    pub const C3270: u8 = 253;
    pub const XKB: u8 = 254;
    pub const KEYBOARD: u8 = 255;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keysym {
    value: u32,
}

impl Keysym {
    pub fn new(value: u32) -> Keysym {
        Keysym { value }
    }

    /// get the character representation of a keysym
    pub fn character(&self) -> Result<char, Error> {
        match ((self.value & 0xff00) >> 8) as u8 {
            CharacterSet::LATIN1 => {
                char::from_u32((self.value & 0xff) + 0x20 - 32).ok_or(Error::InvalidKeysym)
            }
            _ => Err(Error::InvalidKeysym),
        }
    }
}
