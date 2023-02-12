use crate::engines::KvsEngine;
use std::net::{SocketAddr, TcpListener};
use crate::error::{KVError, Result};

pub struct Server<E: KvsEngine> {
    engine: E,
    addr: SocketAddr,
    listener: TcpListener,
}

impl<E: KvsEngine> Server<E> {

    pub fn new(engine: E, addr: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            engine,
            addr,
            listener
        })
    }

    pub fn run(&self) -> Result<()> {
        todo!()
        // for stream in self.listener.incoming() {

        // }
    }
}