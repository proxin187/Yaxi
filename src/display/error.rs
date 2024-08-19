

pub enum Error {
    InvalidStatus,
    SetupFailed,
    Authenthicate,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidStatus => {
                f.write_str("server responded with invalid status code")
            },
            Error::SetupFailed => {
                f.write_str("connection initiation setup failed")
            },
            Error::Authenthicate => {
                f.write_str("authenthication required")
            },
        }
    }
}

impl std::error::Error for Error {}


