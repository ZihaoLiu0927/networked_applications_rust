use std::{process};
use kvs::parser::client_parser;
use std::env::current_dir;
use kvs::Result;


fn main() -> Result<()> {

    let cli = client_parser::Cli::parse_cli();
    dbg!(cli);


    // let mut kvs = KvStore::open(current_dir()?)?;
    // match cli.params {
    //     Methods::Set(action) => {
    //         kvs.set(String::from(action.key), String::from(action.value))?;
    //     },
    //     Methods::Get(action) => {
    //         if let Some(res) = kvs.get(String::from(action.key))? {
    //             println!("{}", res);
    //         } else {
    //             println!("Key not found");
    //         }
    //     }
    //     Methods::Rm(action) => {
    //         kvs.remove(String::from(action.key)).unwrap_or_else(|_| {
    //             println!("Key not found");
    //             process::exit(1);
    //         });
    //     }
    // }
    Ok(())
}