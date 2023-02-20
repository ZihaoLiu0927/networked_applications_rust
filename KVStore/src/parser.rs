use crate::{
    common::{Engine, Methods},
};
use clap::{self, Parser};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::Kvs;


// used by kvs-client to parse command line parameters
pub mod client_parser {
    use super::*;

    #[derive(Parser, Debug)]
    #[clap(author = env!("CARGO_PKG_AUTHORS"), 
           version = env!("CARGO_PKG_VERSION"), 
           about = env!("CARGO_PKG_DESCRIPTION"), 
           name = env!("CARGO_PKG_NAME"))]
    pub struct Cli {
        #[clap(subcommand)]
        pub params: Methods,
    }

    impl Cli {
        pub fn parse_cli() -> Self {
            Self::parse()
        }
    }

}


// used by kvs-server to parse command line parameters
pub mod server_parser {
    use super::*;

    #[derive(Parser, Debug)]
    #[clap(author = env!("CARGO_PKG_AUTHORS"), 
           version = env!("CARGO_PKG_VERSION"), 
           about = env!("CARGO_PKG_DESCRIPTION"), 
           name = env!("CARGO_PKG_NAME"))]
    pub struct Cli {
        #[arg(short, long, default_value_t = String::from(super::DEFAULT_LISTENING_ADDRESS))]
        pub addr: String,
        #[arg(value_enum, short, long, default_value_t = super::DEFAULT_ENGINE)]
        pub engine: Engine,
    }

    impl Cli {
        pub fn parse_cli() -> Self {
            Self::parse()
        }
    }

}