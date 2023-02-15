use std::net::SocketAddr;
use kvs::{
    Result,
    parser::client_parser, client::Client, 
    common::{Methods, GetAction, SetAction, RemoveAction}
};


fn main() -> Result<()> {

    let cli = client_parser::Cli::parse_cli();

    // let socket: SocketAddr = cli.addr.parse()?;

    // let mut client = Client::new(socket)?;

    match cli.params {
        Methods::Get(GetAction{key, addr}) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            let response = client.get(key)?;
            println!("{}", response);
        }
        Methods::Set(SetAction{key, value, addr}) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            client.set(key, value)?;
        }
        Methods::Rm(RemoveAction {key, addr}) => {
            let socket: SocketAddr = addr.parse()?;
            let mut client = Client::new(socket)?;
            client.remove(key)?;

        }
    }

    Ok(())
}