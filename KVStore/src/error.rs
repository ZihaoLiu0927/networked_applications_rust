use failure::Fail;
use std::io;

#[derive(Debug, Fail)]
pub enum KVError {
    #[fail(display = "io error")]
    Io(),
    
    #[fail(display = "serde error")]
    Serde,

    #[fail(display = "Key not found!")]
    KeyNoExist,

    #[fail(display = "log inconsistency")]
    LogInConsistency,
}


impl From<serde_json::Error> for KVError {
    fn from(_err: serde_json::Error) -> KVError { 
        KVError::Serde
    }
}

impl From<io::Error> for KVError {
    fn from(_err: io::Error) -> KVError {
        KVError::Io()
    }
}

pub type Result<T> = std::result::Result<T, KVError>;