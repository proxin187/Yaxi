use super::ErrorCode;


pub enum Error {
    InvalidOpcode,
    InvalidStatus,
    InvalidId,
    InvalidAtom,
    InvalidKeysym,
    Authenthicate,
    RanOutOfXid,
    NoScreens,
    NoReply,
    FailedToLock,
    Stream,
    Utf8,
    Spmc,
    SetupFailed {
        reason: String,
    },
    Event {
        error: ErrorCode,
        major_opcode: u8,
        minor_opcode: u16,
        bad_value: u32,
        sequence: u16,
    },
    Other {
        error: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidOpcode => {
                f.write_str("server sendt invalid opcode for event")
            },
            Error::InvalidStatus => {
                f.write_str("server responded with invalid status code")
            },
            Error::InvalidId => {
                f.write_str("invalid id")
            },
            Error::InvalidAtom => {
                f.write_str("invalid atom")
            },
            Error::InvalidKeysym => {
                f.write_str("invalid keysym")
            },
            Error::Authenthicate => {
                f.write_str("authenthication required")
            },
            Error::RanOutOfXid => {
                f.write_str("server ran out of xid's")
            },
            Error::NoScreens => {
                f.write_str("server never informed of any screens")
            },
            Error::NoReply => {
                f.write_str("reply queue empty")
            },
            Error::FailedToLock => {
                f.write_str("failed to lock mutex")
            },
            Error::Stream => {
                f.write_str("stream failed")
            },
            Error::Utf8 => {
                f.write_str("invalid utf8")
            },
            Error::Spmc => {
                f.write_str("singe-producer multi-consumer implementation error")
            },
            Error::SetupFailed { reason } => {
                f.write_fmt(format_args!("connection initiation setup failed: {}", reason))
            },
            Error::Event { error, major_opcode, minor_opcode, bad_value, sequence } => {
                f.write_fmt(format_args!("error={:?}, major_opcode={}, minor_opcode={}, bad_value={}, sequence={}", error, major_opcode, minor_opcode, bad_value, sequence))
            },
            Error::Other { error } => {
                f.write_fmt(format_args!("other: {}", error))
            },
        }
    }
}

impl std::error::Error for Error {}


