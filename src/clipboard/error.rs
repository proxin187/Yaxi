use crate::display;

#[derive(Debug)]
pub enum Error {
    ServiceStopped,
    HandoverError,
    Terminated,
    EventLoopError(String),
    EventLoopStopped,
    InvalidData(String),
    FailedToAcquireOwnership,
    SelectionTimeout,
    SelectionNoData,
    InvalidProperty,
    Timeout,
    SaveFailed,
    ConversionFailure,
    FailedToLock,
    NoManager,
    FromUtf8Error(std::string::FromUtf8Error),
    Display(display::error::Error),
    RwLock(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ServiceStopped => write!(f, "ServiceStopped"),
            Error::HandoverError => write!(f, "HandoverError"),
            Error::Terminated => write!(f, "Terminated"),
            Error::EventLoopError(e) => write!(f, "EventLoopError: {}", e),
            Error::EventLoopStopped => write!(f, "EventLoopStopped"),
            Error::InvalidData(e) => write!(f, "InvalidData: {}", e),
            Error::FailedToAcquireOwnership => write!(f, "FailedToAcquireOwnership"),
            Error::SelectionTimeout => write!(f, "SelectionTimeout"),
            Error::SelectionNoData => write!(f, "SelectionNoData"),
            Error::InvalidProperty => write!(f, "InvalidProperty"),
            Error::Timeout => write!(f, "Timeout"),
            Error::SaveFailed => write!(f, "SaveFailed"),
            Error::ConversionFailure => write!(f, "ConversionFailure"),
            Error::FailedToLock => write!(f, "FailedToLock"),
            Error::NoManager => write!(f, "NoManager"),
            Error::FromUtf8Error(e) => write!(f, "FromUtf8Error: {}", e),
            Error::Display(e) => write!(f, "Display: {}", e),
            Error::RwLock(e) => write!(f, "RwLock: {}", e),
            Error::Other(e) => write!(f, "Other: {}", e),
        }
    }
}

impl From<display::error::Error> for Error {
    fn from(e: display::error::Error) -> Self {
        Error::Display(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(e)
    }
}

impl std::error::Error for Error {}
