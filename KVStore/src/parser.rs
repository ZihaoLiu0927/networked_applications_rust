pub mod cli_parser {
    use clap::{Parser, Subcommand};
    use serde::{Serialize, Deserialize};

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
}