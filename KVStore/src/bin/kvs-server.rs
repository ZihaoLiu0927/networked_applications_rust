use kvs::{Result};
use kvs::parser::server_parser;
use std::net::SocketAddr;
use std::env::current_dir;
use std::fs::{self};
use std::sync::Mutex;

use kvs::error::{KVError};
use kvs::common::*;
use kvs::{KvStore, SledKvsEngine, KvsEngine};
use kvs::server::Server;


extern crate slog;
extern crate slog_term;
extern crate slog_json;
extern crate slog_async;

use crate::slog::{info, Drain};

const ENGINE_KVS: &str = "kvs";
const ENGINE_SLED: &str = "sled";


fn main() -> Result<()> {

    let drain = slog_term::CompactFormat::new(
        slog_term::PlainSyncDecorator::new(std::io::stderr()))
        .build()
        .fuse();

    let root_logger = slog::Logger::root(Mutex::new(drain).fuse(), slog::o!());

    let cli = server_parser::Cli::parse_cli();

    let engine = access_engine()?
        .unwrap_or(cli.engine);
    let socket: SocketAddr = cli.addr.parse()?; 

    run(engine, socket)?;

    Ok(())

    // let listener = TcpListener::bind(socket).unwrap();
    // for stream in listener.incoming() {
    //     let a = stream?;
    // }
 }

fn run(engine: Engine, addr: SocketAddr) -> Result<()> {
    fs::write(current_dir()?.join("engine"), format!("{}", engine))?;

    match engine {
        Engine::Kvs => {
            let engine = KvStore::open(current_dir()?)?;
            run_server(engine, addr)?;
        }
        Engine::Sled => {
            let engine = SledKvsEngine::open(current_dir()?)?;
            run_server(engine, addr)?;
        }
    };

    Ok(())
}


fn run_server<E: KvsEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    let server = Server::new(engine, addr)?;
    server.run()?;
    Ok(())
}

// the parser ensures that input engine must be either kvs or sled
// check_engine return an existing engine or create a new engine record file 
fn access_engine() -> Result<Option<Engine>> {
    let path = current_dir()?.join("engine");
    if path.exists() {

        let content = fs::read_to_string(path)?.to_lowercase();

        match content.as_str() {
            ENGINE_KVS => return Ok(Some(Engine::Kvs)),
            ENGINE_SLED => return Ok(Some(Engine::Sled)),
            _ => return Err(KVError::EngineNotMatch),
        }
        
    }
    Ok(None)
}