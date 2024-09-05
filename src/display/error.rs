

pub enum Error {
    InvalidOpcode,
    InvalidStatus,
    Authenthicate,
    RanOutOfXid,
    NoScreens,
    InvalidId,
    NoReply,
    SetupFailed {
        reason: String,
    },
    Event {
        detail: u8,
        sequence: u16,
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
            Error::Authenthicate => {
                f.write_str("authenthication required")
            },
            Error::RanOutOfXid => {
                f.write_str("server ran out of xid's")
            },
            Error::NoScreens => {
                f.write_str("server never informed of any screens")
            },
            Error::InvalidId => {
                f.write_str("invalid id")
            },
            Error::NoReply => {
                f.write_str("reply queue empty")
            },
            Error::SetupFailed { reason } => {
                f.write_fmt(format_args!("connection initiation setup failed: {}", reason))
            },
            Error::Event { detail, sequence } => {
                f.write_fmt(format_args!("[error event] detail={}, sequence={}", detail, sequence))
            },
        }
    }
}

impl std::error::Error for Error {}


