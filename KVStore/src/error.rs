use failure::Fail;
use slog::KV;
use std::{io, net, fmt};

#[derive(Debug, Fail)]
pub enum KVError {
    #[fail(display = "Error: an io error happened!")]
    Io,
    
    #[fail(display = "Error: a serde error happened!")]
    Serde,

    #[fail(display = "Error: Key not found!")]
    KeyNoExist,

    #[fail(display = "Error: log inconsistency!")]
    LogInConsistency,

    #[fail(display = "Error: a format error happened!")]
    Fmt,

    #[fail(display = "Error: invalid ip or port address")]
    InvalidIpOrPort,

    #[fail(display = "Error: assigned engine does not match existing one!")]
    EngineNotMatch,

    #[fail(display = "Error: cannot parse the string")]
    ParseError,

    #[fail(display = "Error: display trait error happened!")]
    DisplayError,
}


impl From<serde_json::Error> for KVError {
    fn from(_err: serde_json::Error) -> KVError { 
        KVError::Serde
    }
}

impl From<io::Error> for KVError {
    fn from(_err: io::Error) -> KVError {
        KVError::Io
    }
}

impl From<net::AddrParseError> for KVError {
    fn from(_err: net::AddrParseError) -> KVError {
        KVError::InvalidIpOrPort
    }
}

impl From<fmt::Error> for KVError {
    fn from(_err: fmt::Error) -> KVError {
        KVError::DisplayError
    }
}

pub type Result<T> = std::result::Result<T, KVError>;