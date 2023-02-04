use std::collections::HashMap;

pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, val: String) {
        self.map.insert(key, val);
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}

pub mod cli_parser {
    use clap::{Parser, Subcommand};

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

    #[derive(Subcommand, Debug)]
    pub enum Methods {
        Set(SetAction),
        Get(GetAction),
        Rm(RemoveAction),
    }

    #[derive(Debug, Parser)]
    pub struct SetAction {
        #[clap(index = 1)]
        pub key: String,
        #[clap(index = 2)]
        pub value: String,
    }

    #[derive(Debug, Parser)]
    pub struct GetAction {
        #[clap(index = 1)]
        pub key: String,
    }

    #[derive(Debug, Parser)]
    pub struct RemoveAction {
        #[clap(index = 1)]
        pub key: String,
    }
}
