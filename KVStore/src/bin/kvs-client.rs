use kvs::{
    client::Client,
    common::{GetAction, Methods, RemoveAction, SetAction},
    parser::client_parser,
    Result,
};
use std::net::SocketAddr;

fn main() -> Result<()> {
    let cli = client_parser::Cli::parse_cli();

    match cli.params {
        Methods::Get(GetAction { key, addr }) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            let response = client.get(key)?;
            println!("{}", response);
        }
        Methods::Set(SetAction { key, value, addr }) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            client.set(key, value)?;
        }
        Methods::Rm(RemoveAction { key, addr }) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            client.remove(key)?;
        }
    }

    Ok(())
}
