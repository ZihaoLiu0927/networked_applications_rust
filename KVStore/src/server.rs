use serde_json::Deserializer;
use crate::{
    error::Result,
    common::Request,
    thread_pool::*,
    engines::KvsEngine, common::{GetResponse, SetResponse, RmResponse}
};
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    io::{BufReader, BufWriter, Write},
    sync::{Arc, atomic::{AtomicBool, Ordering::SeqCst}},
};

// Server is a runable server instance with pluggale engine
pub struct Server<E: KvsEngine, P: ThreadPool> {
    engine: E,
    listener: TcpListener,
    pool: P,
    killed: Arc<AtomicBool>,
}

// Server is a runable server instance with pluggale engine
// implement a new() and run() method to it
impl<E: KvsEngine, P: ThreadPool> Server<E, P> {

    // create a new Server instance
    pub fn new(engine: E, addr: SocketAddr, pool: P, killed: Arc<AtomicBool>) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            engine,
            listener,
            pool,
            killed,
        })
    }

    // run starts to listen a port and response any requests from client side
    pub fn run(&mut self) -> Result<()> {

        let listener = self.listener.try_clone()?;

        for stream in listener.incoming() {

            if self.killed.load(SeqCst) {
                break;
            }

            let engine = self.engine.clone();

            match stream {
                Ok(stream) => {
                    self.pool.spawn(move || {
                        if let Err(e) =  request_handler(engine, stream) {
                            eprintln!("Error in request handling: {}", e);
                        }
                    });
                }

                Err(e) => {
                    eprintln!("connection error: {}", e);
                }
            }
        }
        Ok(())
    }

}


fn request_handler<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let reader = Deserializer::from_reader(BufReader::new(stream.try_clone()?));
    let mut writer = BufWriter::new(stream);

    let mut reader_decode = reader.into_iter::<Request>();

    let req = reader_decode.next().expect("client request should not be empty!")?;

    let serialized = match req {
        Request::Get {key} => {
            let get_res = match engine.get(key) {
                Ok(content) =>  GetResponse::Ok(content),
                Err(e) => GetResponse::Err(e.to_string()),
            };
            serde_json::to_string_pretty(&get_res)?
        },
        Request::Set {key, value} => {
            let set_res = match engine.set(key, value) {
                Ok(_) => SetResponse::Ok(),
                Err(e) => SetResponse::Err(e.to_string()),
            };
            serde_json::to_string_pretty(&set_res)?
        },
        Request::Remove {key} => {
            let rm_res = match engine.remove(key) {
                Ok(_) => RmResponse::Ok(),
                Err(e) => RmResponse::Err(e.to_string()),
            };
            serde_json::to_string_pretty(&rm_res)?
        },
    };

    writer.write(serialized.as_bytes())?;
    writer.flush()?;

    Ok(())
}