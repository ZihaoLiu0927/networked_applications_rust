use std::{collections::HashMap};
use std::path::{PathBuf, Path};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, Read, Seek, SeekFrom};
use serde_json::Deserializer;

use failure::{Error, Fail, format_err};
use cli_parser::{Methods, SetAction, RemoveAction};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum KVError {
    #[fail(display = "Error: {}", name)]
    RemoveKeyNoExist {
        name: String,
    }
}

pub struct KvStore {
    diskmap: HashMap<String, u64>,
    reader: KVDiskReader<File>,
    writer: KVDiskWriter<File>,
}

impl KvStore {

    pub fn new(file: File) -> Result<Self> {
        
        let mut kvs = Self {
            diskmap: HashMap::new(),
            reader: KVDiskReader::new(file.try_clone()?)?,
            writer: KVDiskWriter::new(file)?,
        };

        kvs.recover_from_disk()?;
        Ok(kvs)
    }

    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let command = Methods::Set(SetAction {
            key: key.clone(), 
            value: val.clone(),
        });
        let serialized = serde_json::to_string_pretty(&command)?;
        let curr_offset = self.writer.write(serialized)?;
        self.diskmap.insert(key, curr_offset);
        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let offset;
        if let Some(val) = self.diskmap.get(&key) {
            offset = *val;
        } else {
            return Ok(None);
        }

        match self.reader.read(offset) {
            Ok(s) => Ok(Some(s)),
            Err(_) => Ok(None),
        }        
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if let Some(_) =  self.diskmap.get(&key) {
            let command = Methods::Rm(RemoveAction {
                key: key.clone(), 
            });
            let serialized = serde_json::to_string_pretty(&command)?;
            self.writer.write(serialized)?;
            self.diskmap.remove(&key);
        } else {
            return Err(format_err!("Key not found!"));
        }
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let file = OpenOptions::new().
                            create(true).
                            write(true).
                            read(true).
                            append(true).
                            open(create_log_filename(&path.into()))?;
        let kvs = KvStore::new(file);
        kvs
    }

    fn recover_from_disk(&mut self) -> Result<()> {
        let map = &mut self.diskmap;
        let lastest_cursor = self.reader.load_log_from_disk(map)?;
        self.writer.update_disk_cursor(lastest_cursor)
    }

}

struct KVDiskReader<R: Read+Seek> {
    reader: BufReader<R>,
}

impl<R: Read+Seek> KVDiskReader<R> {
    pub fn new(inner: R) -> Result<Self> {
        let reader = BufReader::new(inner);
        Ok(
            Self {
            reader: reader,
        })
    }

    pub fn read(&mut self, offset: u64) -> Result<String> {
        let ref_reader = self.reader.get_mut();
        ref_reader.seek(SeekFrom::Start(offset))?;
        let mut stream = Deserializer::from_reader(ref_reader)
                                                                .into_iter::<Methods>();
        if let Some(cmd) = stream.next() {
            match cmd? {
                Methods::Set(action) => {
                    return Ok(action.value);
                }
                Methods::Rm(_) => {
                    return Err(format_err!("Reading a remove operation should not happen!"));
                }
                _ => {
                    return Err(format_err!("this should not happen!"));
                }
            }
        } 
        return Err(format_err!("this should not happen!"));
    }

    pub fn load_log_from_disk(&mut self, map: &mut HashMap<String, u64>) -> Result<u64> {
        let ref_reader = self.reader.get_mut();
        ref_reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(ref_reader).
                                                                into_iter::<Methods>();
        let mut old_offset;
        loop {
            old_offset = stream.byte_offset() as u64;
            if let Some(cmd) = stream.next() {
                match cmd? {
                    Methods::Set(action) => {
                        map.insert(action.key, old_offset);
                    }
                    Methods::Rm(action) => {
                        map.remove(&action.key);
                    }
                    _ => {}
                }
                continue
            } 
            return  Ok(old_offset);
        };
    }
}

struct KVDiskWriter<W: Write> {
    writer: BufWriter<W>,
    cursor: u64
}

impl<W: Write> KVDiskWriter<W> {
    pub fn new(inner: W) -> Result<Self> {
        let writer = BufWriter::new(inner);
        Ok(
            Self {
            writer: writer,
            cursor: 0,
        })
    }
    
    pub fn write(&mut self, serialized: String) -> Result<u64> {
        let len = self.writer.write(serialized.as_bytes())?;
        let old_offset = self.cursor;
        self.cursor = old_offset + len as u64;
        self.writer.flush()?;
        Ok(old_offset)
    }

    fn update_disk_cursor(&mut self, offset: u64) -> Result<()> {
        self.cursor = offset;
        Ok(())
    }
}


pub mod cli_parser {
    use clap::{Parser, Subcommand};
    use serde::{Serialize, Deserialize};

    #[derive(Parser, Debug)]
    #[clap(author = env!("CARGO_PKG_AUTHORS"), 
           version = env!("CARGO_PKG_VERSION"), 
           about = env!("CARGO_PKG_DESCRIPTION"), 
           name = env!("CARGO_PKG_NAME"))]
    pub struct Cli {
        #[clap(subcommand)]
        pub params: Methods,
    }

    impl Cli {
        pub fn parse_cli() -> Self {
            Self::parse()
        }
    }

    #[derive(Subcommand, Debug, Serialize, Deserialize)]
    pub enum Methods {
        Set(SetAction),
        Get(GetAction),
        Rm(RemoveAction),
    }

    #[derive(Debug, Parser, Serialize, Deserialize)]
    pub struct SetAction {
        #[clap(index = 1)]
        pub key: String,
        #[clap(index = 2)]
        pub value: String,
    }

    #[derive(Debug, Parser, Serialize, Deserialize)]
    pub struct GetAction {
        #[clap(index = 1)]
        pub key: String,
    }

    #[derive(Debug, Parser, Serialize, Deserialize)]
    pub struct RemoveAction {
        #[clap(index = 1)]
        pub key: String,
    }
}

fn create_log_filename(dir_path: &Path) -> PathBuf {
    let ret = dir_path.join(format!("KVStore.log"));
    return ret;
}