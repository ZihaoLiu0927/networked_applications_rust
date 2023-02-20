use slog::Logger;

use kvs::{
    Result,
    KvStore, 
    SledKvsEngine, 
    KvsEngine,
    parser::server_parser,
    error::KVError,
    common::*,
    server::Server, 
    ThreadPool, 
    thread_pool::RayonThreadPool,
};
use std::{
    fs,
    net::SocketAddr,
    sync::{Arc, Mutex, atomic::AtomicBool},
    env::current_dir,
};

extern crate slog;
extern crate slog_term;
extern crate slog_json;
extern crate slog_async;
use crate::slog::Drain;

const ENGINE_KVS: &str = "kvs";
const ENGINE_SLED: &str = "sled";
const ENGINE_FILE: &str = "engine.rec";
const ENGINE_DB_DI: &str = "database";


fn main() -> Result<()> {

    let drain = slog_term::CompactFormat::new(
        slog_term::PlainSyncDecorator::new(std::io::stderr()))
        .build()
        .fuse();

    let root_logger = slog::Logger::root(Mutex::new(drain).fuse(), slog::o!());

    let cli = server_parser::Cli::parse_cli();

    let engine = check_engine(cli.engine)?;

    let socket: SocketAddr = cli.addr.parse()?; 

    run(engine, socket, root_logger)?;

    Ok(())

 }

fn run(engine: Engine, addr: SocketAddr, logger: Logger) -> Result<()> {

    slog::info!(logger, ""; "kv server" => env!("CARGO_PKG_VERSION"));
    slog::info!(logger, ""; "ip" => format!("{}:{}", addr.ip(), addr.port()));
    slog::info!(logger, ""; "Engine" => format!("{}", engine));

    fs::write(current_dir()?.join(ENGINE_FILE), format!("{}", engine))?;

    let pool = RayonThreadPool::new(8)?;

    match engine {
        Engine::Kvs => {
            let engine = KvStore::open(current_dir()?.join(ENGINE_DB_DI))?;
            run_kv_server(engine, addr, pool)?;
        }
        Engine::Sled => {
            let engine = SledKvsEngine::open(current_dir()?.join(ENGINE_DB_DI))?;
            run_kv_server(engine, addr, pool)?;
        }
    };

    Ok(())
}


fn run_kv_server<E: KvsEngine, P: ThreadPool>(engine: E, addr: SocketAddr, pool: P) -> Result<()> {
    let killed = Arc::new(AtomicBool::new(false)); 
    let mut server = Server::new(engine, addr, pool, killed)?;
    server.run()?;
    Ok(())
}

// the parser ensures that input engine must be either kvs or sled
// check_engine return an existing engine or create a new engine record file 
fn check_engine(assigned_engine: Engine) -> Result<Engine> {
    let path = current_dir()?.join(ENGINE_FILE);
    if path.exists() {

        let content = fs::read_to_string(path)?.to_lowercase();

        let read_engine = match content.as_str() {
            ENGINE_KVS => Engine::Kvs,
            ENGINE_SLED => Engine::Sled,
            _ => return Err(KVError::EngineNotMatch),
        };
        
        if read_engine == assigned_engine {
            return Ok(assigned_engine);
        }

        return Err(KVError::EngineNotMatch)
    }

    Ok(assigned_engine)
}