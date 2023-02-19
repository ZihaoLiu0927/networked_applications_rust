use crate::{engines::KvsEngine, common::{GetResponse, SetResponse, RmResponse}};
use crate::error::Result;
use crate::common::Request;
use crate::thread_pool::*;

use serde_json::Deserializer;

use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    io::{BufReader, BufWriter, Write},
};

// Server is a runable server instance with pluggale engine
pub struct Server<E: KvsEngine> {
    engine: E,
    listener: TcpListener,
}

// Server is a runable server instance with pluggale engine
// implement a new() and run() method to it
impl<E: KvsEngine> Server<E> {

    // create a new Server instance
    pub fn new(engine: E, addr: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            engine,
            listener
        })
    }

    // run starts to listen a port and response any requests from client side
    pub fn run(&mut self) -> Result<()> {

        let thread_pool = NaiveThreadPool::new(6)?;

        let listener = self.listener.try_clone()?;

        for stream in listener.incoming() {

            let engine = self.engine.clone();

            let stream = stream?;
            thread_pool.spawn(move || {
                if let Err(e) =  request_handler(engine, stream) {
                    eprintln!("Error in request handling: {}", e);
                }
            });
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