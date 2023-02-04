use kvs::cli_parser::*;
use std::process;

fn main() {
    //let mut kvs = KvStore::new();

    let cli = Cli::parse_cli();

    match cli.params {
        // Methods::Set(action) => kvs.set(String::from(action.key), String::from(action.value)),
        // Methods::Get(action) => {
        //     if let Some(res) = kvs.get(String::from(action.key)) {
        //         println!("Retrived value {}", res);
        //     }
        // }
        // Methods::Remove(action) => kvs.remove(String::from(action.key)),
        Methods::Set(action) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        Methods::Get(action) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        Methods::Rm(action) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
    }
}
