use std::{process};
use kvs::{KvStore, Result};
use kvs::cli_parser::*;
use std::env::current_dir;

fn main() -> Result<()> {

    let cli = Cli::parse_cli();

    let mut kvs = KvStore::open(current_dir()?)?;

    match cli.params {
        Methods::Set(action) => {
            kvs.set(String::from(action.key), String::from(action.value))?;
        },
        Methods::Get(action) => {
            if let Some(res) = kvs.get(String::from(action.key))? {
                print!("{}", res);
            } else {
                println!("Key not found");
            }
        }
        Methods::Rm(action) => {
            kvs.remove(String::from(action.key)).unwrap_or_else(|_| {
                println!("Key not found");
                process::exit(1);
            });
        }
    }
    Ok(())
}
