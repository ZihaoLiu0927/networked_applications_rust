use clap::{self, Subcommand, Parser};
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::fmt::{self,Display};

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
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct GetAction {
    #[clap(index = 1)]
    pub key: String,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct RemoveAction {
    #[clap(index = 1)]
    pub key: String,
}



#[derive(clap::ValueEnum, Clone, Debug, Parser)]
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