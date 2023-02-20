use crate::common::{Request, GetResponse, SetResponse, RmResponse};
use crate::error::{KVError,Result};

use serde::Deserialize;
use serde_json::{Deserializer, de::IoRead};

use std::{
    net::{SocketAddr, TcpStream},
    io::{BufReader, BufWriter, Write}
};

pub struct Client {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}


impl Client {
    pub fn new(addr: SocketAddr) -> Result<Self> {

        let stream = TcpStream::connect(addr).expect("error in connection!");
        let reader = BufReader::new(stream.try_clone().expect("error in clone stream"));

        Ok(Self {
            reader: Deserializer::from_reader(reader),
            writer: BufWriter::new(stream)
        })
    }

    pub fn get(&mut self, key: String) -> Result<String> {
        let serialized = serde_json::to_string_pretty(&Request::Get{key})?;
        self.writer.write(serialized.as_bytes())?;
        self.writer.flush()?;

        let response = GetResponse::deserialize(&mut self.reader)?;

        match response {
            GetResponse::Ok(content) => {
                Ok(content.unwrap_or(KVError::KeyNoExist.to_string()))
            }
            GetResponse::Err(e) => {
                Err(KVError::String(e))
            }
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&Request::Set{key, value})?;
        self.writer.write(serialized.as_bytes())?;
        self.writer.flush()?;

        let response = SetResponse::deserialize(&mut self.reader)?;

        match response {
            SetResponse::Ok() => {
                Ok(())
            }
            SetResponse::Err(e) => {
                Err(KVError::String(e))
            }
        }
        
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let serialized = serde_json::to_string_pretty(&Request::Remove{key})?;
        self.writer.write(serialized.as_bytes())?;
        self.writer.flush()?;

        let response = RmResponse::deserialize(&mut self.reader)?;

        match response {
            RmResponse::Ok() => {
                Ok(())
            }
            RmResponse::Err(e) => {
                Err(KVError::String(e))
            }
        }
    }


}
