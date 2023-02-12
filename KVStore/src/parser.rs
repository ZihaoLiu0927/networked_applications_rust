
pub mod client_parser {
    use clap::{self, Parser};
    use crate::common::Methods;

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


pub mod server_parser {

    const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
    const DEFAULT_ENGINE: Engine = Engine::Kvs;

    use clap::{Parser};
    use crate::common::Engine;

    #[derive(Parser, Debug)]
    #[clap(author = env!("CARGO_PKG_AUTHORS"), 
           version = env!("CARGO_PKG_VERSION"), 
           about = env!("CARGO_PKG_DESCRIPTION"), 
           name = env!("CARGO_PKG_NAME"))]
    pub struct Cli {
        #[arg(short, long, default_value_t = String::from(DEFAULT_LISTENING_ADDRESS))]
        pub addr: String,
        #[arg(value_enum, short, long, default_value_t = DEFAULT_ENGINE)]
        pub engine: Engine,
    }

    impl Cli {
        pub fn parse_cli() -> Self {
            Self::parse()
        }
    }

}