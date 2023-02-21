use failure::Fail;
use std::{fmt, io, net, str::Utf8Error};

// error handling. Any error will be converted to the same type: KVError
// to facilitate the development
#[derive(Debug, Fail)]
pub enum KVError {
    #[fail(display = "")]
    String(String),

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

    #[fail(display = "Error: cannot create client instance!")]
    FailClientInstance,

    #[fail(display = "Error: from client request.")]
    RequestError,

    #[fail(display = "Error: sled error")]
    SledError,

    #[fail(display = "Error: get method error!")]
    GetMethodError,
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

impl From<sled::Error> for KVError {
    fn from(_err: sled::Error) -> KVError {
        KVError::SledError
    }
}

impl From<Utf8Error> for KVError {
    fn from(_err: Utf8Error) -> KVError {
        KVError::SledError
    }
}

pub type Result<T> = std::result::Result<T, KVError>;
