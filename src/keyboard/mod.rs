

// page 86 @ https://www.x.org/docs/XProtocol/proto.pdf


#[derive(Clone, Copy)]
pub enum CharacterSet {
    Latin1 = 0,
    Latin2 = 1,
    Latin3 = 2,
    Latin4 = 3,
    Kana = 4,
    Arabic = 5,
    Cyrillic = 6,
    Greek = 7,
    Technical = 8,
    Special = 9,
    Publishing = 10,
    Apl = 11,
    Hebrew = 12,
    Thai = 13,
    Korean = 14,
    Latin5 = 15,
    Latin6 = 16,
    Latin7 = 17,
    Latin8 = 18,
    Latin9 = 19,
    Currency = 32,
    C3270 = 253,
    Xkb = 254,
    Keyboard = 255,
}

#[derive(Clone, Copy)]
pub enum KeyMask {
    Shift = 0x0001,
    Lock = 0x0002,
    Control = 0x0004,
    Mod1 = 0x0008,
    Mod2 = 0x0010,
    Mod3 = 0x0020,
    Mod4 = 0x0040,
    Mod5 = 0x0080,
    Button1 = 0x0100,
    Button2 = 0x0200,
    Button3 = 0x0400,
    Button4 = 0x0800,
    Button5 = 0x1000,
}

#[derive(Clone, Copy)]
pub enum Keycode {
    Any,
}


