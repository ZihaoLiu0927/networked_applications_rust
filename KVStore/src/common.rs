use clap::{self, Subcommand, Parser};
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::fmt::{self,Display};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum Methods {
    Set(SetAction),
    Get(GetAction),
    Rm(RemoveAction),
}


#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct SetAction {
    #[clap(index = 1)]
    pub key: String,
    #[clap(index = 2)]
    pub value: String,
    #[arg(short, long, default_value_t = String::from(DEFAULT_LISTENING_ADDRESS))]
    pub addr: String,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct GetAction {
    #[clap(index = 1)]
    pub key: String,
    #[arg(short, long, default_value_t = String::from(DEFAULT_LISTENING_ADDRESS))]
    pub addr: String,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct RemoveAction {
    #[clap(index = 1)]
    pub key: String,
    #[arg(short, long, default_value_t = String::from(DEFAULT_LISTENING_ADDRESS))]
    pub addr: String,
}



#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get {key: String},
    Set {key: String, value: String},
    Remove {key: String},
}


#[derive(Debug, Serialize, Deserialize)]
pub enum GetResponse {
    Ok(Option<String>),
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SetResponse {
    Ok(),
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RmResponse {
    Ok(),
    Err(String),
}



#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum Engine {
    Kvs,
    Sled,
}

impl Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> { 
        match self {
            Engine::Kvs => {
                return Ok(write!(f, "{}", "kvs")?);
            }
            Engine::Sled => {
                return Ok(write!(f, "{}", "sled")?);
            }
        }
    }
}



// extenable for future
#[macro_export]
macro_rules! logfile {
    ($dir: expr, $fgen: expr) => {
        $dir.join(format!("{}.log", $fgen))
    }
}