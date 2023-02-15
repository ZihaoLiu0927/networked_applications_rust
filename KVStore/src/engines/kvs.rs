use crate::KvsEngine;
use crate::error::{KVError, Result};
use crate::logfile;

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use std::{
    ffi::OsStr,
    collections::HashMap,
    path::{PathBuf, Path},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write, SeekFrom, Seek, BufReader, BufWriter},
};


const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1MB

pub struct KvStore {
    path: PathBuf,
    indexmap: HashMap<String, DiskPos>,
    readers: HashMap<u64, KVDiskReader<File>>,
    writer: KVDiskWriter<File>,
    curr_gen: u64,
    need_compact: u64,
}

impl KvStore {

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut indexmap: HashMap<String, DiskPos> = HashMap::new();
        let mut need_compact = 0;

        let mut readers: HashMap<u64, KVDiskReader<File>> = HashMap::new(); 
        let gen_list = collect_file_identifiers(&path)?;

        for &gen in &gen_list {
            let mut reader = KVDiskReader::new(File::open(logfile!(path, gen))?)?;
            need_compact += reader.load_log_from_disk(&mut indexmap, gen)?;
            readers.insert(gen, reader);
        }

        let curr_gen = gen_list.last().unwrap_or(&0) + 1;

        let writer = Self::create_new_log(&path, curr_gen, &mut readers)?;

        Ok(KvStore {
            path,
            indexmap,
            readers,
            writer,
            curr_gen,
            need_compact,
        })

    }

    fn create_new_log(path: &Path, curr_gen: u64, readers: &mut HashMap<u64, KVDiskReader<File>>) -> Result<KVDiskWriter<File>> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(logfile!(path, curr_gen))?;

        let writer = KVDiskWriter::new(file.try_clone()?)?;
        readers.insert(curr_gen, KVDiskReader::new(file)?);
        

        Ok(writer)
    }

    fn compaction(&mut self) -> Result<()> {

        let gen_compact = self.curr_gen + 1;
        let mut writer = Self::create_new_log(&self.path, gen_compact, &mut self.readers)?;


        let mut pos = 0;
        for entry in &mut self.indexmap.values_mut() {
            let reader = self.readers
                .get_mut(&entry.gen)
                .expect("Cannot find the file reader. This should not happen!");
            if reader.cursor != entry.pos {
                reader.seek(SeekFrom::Start(entry.pos))?;
            }
            let mut read_from = reader.take(entry.len);
            let len = io::copy(&mut read_from, &mut writer)?;
            *entry = (gen_compact, pos, len).into();
            pos += len;
        }


        let gen_newlog = self.curr_gen + 2;
        self.writer = Self::create_new_log(&self.path, gen_newlog, &mut self.readers)?;

        let old_files: Vec<u64> = self.readers
                .keys()
                .filter(|&gen| *gen <= self.curr_gen)
                .map(|x| *x)
                .collect();
        
        for gen in old_files {
            fs::remove_file(logfile!(self.path, gen))?;
            self.readers.remove(&gen);
        }

        self.curr_gen = gen_newlog;
        self.need_compact = 0;
        
        Ok(())
    }

}

impl KvsEngine for KvStore {
    fn remove(&mut self, key: String) -> Result<()> {
        let check = self.get(key.clone())?;
        if check.is_none() {
            return Err(KVError::KeyNoExist)
        }

        if let Some(_) =  self.indexmap.get(&key) {
            let command = Command::Remove { key: key.clone() };
            let serialized = serde_json::to_string_pretty(&command)?;
            let (pos, len) = self.writer.write_entry(serialized)?;
            let diskpos = DiskPos {
                gen: self.curr_gen,
                pos,
                len,
            };
            self.indexmap.insert(key, diskpos);

        } else {
            return Err(KVError::KeyNoExist)
        }
        Ok(())
    }

    fn set(&mut self, key: String, val: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(), 
            value: val.clone(),
        };
        let serialized = serde_json::to_string_pretty(&command)?;
        let (pos, len) = self.writer.write_entry(serialized)?;
        let diskpos = DiskPos {
            gen: self.curr_gen,
            pos,
            len,
        };

        let stale_len = self.indexmap.insert(key, diskpos)
            .unwrap_or(DiskPos::new()).len;
        self.need_compact += stale_len;

        if self.need_compact > COMPACTION_THRESHOLD {
            self.compaction()?;
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(val) = self.indexmap.get(&key) {
            let reader = self.readers
                .get_mut(&val.gen)
                .expect("Cannot find the file reader. This should not happen!");
            
            match reader.read_entry(val.pos, val.len) {
                Ok(s) => Ok(Some(s)),
                Err(_) => Ok(None),
            }   
        } else {
            Ok(None)
        }     
    }
}


fn collect_file_identifiers(dir: &Path) -> Result<Vec<u64>> {
    let mut fgen_list: Vec<u64> = fs::read_dir(&dir)?
        .flat_map(|res| res.and_then(|entry| Ok(entry.path())))
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    
    fgen_list.sort_unstable();
    Ok(fgen_list)
} 


/// Struct representing a command
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}


#[derive(Debug, Clone)]
struct DiskPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl DiskPos {
    pub fn new() -> Self {
        Self {
            gen: 0,
            len: 0,
            pos: 0,
        }
    }
}

impl From<(u64, u64, u64)> for DiskPos {
    fn from((gen, pos, len): (u64, u64, u64)) -> Self { 
        Self {
            gen,
            pos,
            len,
        }
    }
}

struct KVDiskReader<R: Read+Seek> {
    reader: BufReader<R>,
    cursor: u64,
}

impl<R: Read+Seek> KVDiskReader<R> {
    pub fn new(inner: R) -> Result<Self> {
        let reader = BufReader::new(inner);
        Ok(
            Self {
            reader: reader,
            cursor: 0,
        })
    }

    pub fn read_entry(&mut self, offset: u64, len: u64) -> Result<String> {
        let ref_reader = self.reader.get_mut();
        ref_reader.seek(SeekFrom::Start(offset))?;
        let cmd_reader = ref_reader.take(len);

        if let Command::Set {key: _k, value: v} = serde_json::from_reader(cmd_reader)? {
            Ok(v)
        } else {
            Err(KVError::LogInConsistency)
        }
    }

    pub fn load_log_from_disk(&mut self, map: &mut HashMap<String, DiskPos>, fgen: u64) -> Result<u64> {
        let ref_reader = self.reader.get_mut();
        ref_reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(ref_reader).
                                                                into_iter::<Command>();
        let mut pos;
        let mut need_compact = 0;
        loop {
            pos = stream.byte_offset() as u64;
            if let Some(entry) = stream.next() {
                let len = stream.byte_offset() as u64 - pos;
                match entry? {
                    Command::Set{key: k, value: _v} => {
                        let old_entry = map
                            .insert(k, DiskPos { gen: fgen, pos, len })
                            .unwrap_or(DiskPos::new());
                        need_compact += old_entry.len;
                    }
                    Command::Remove{key: k} => {
                        let old_entry = map
                            .insert(k, DiskPos { gen: fgen, pos, len })
                            .unwrap_or(DiskPos::new());
                        need_compact += old_entry.len;
                    }
                }
                continue
            } 
            return Ok(need_compact);
        };

    }

}

impl<R: Read+Seek> Seek for KVDiskReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> { 
        self.cursor = self.reader.seek(pos)?;
        Ok(self.cursor)
    }
}

impl <R: Read+Seek> Read for KVDiskReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.reader.read(buf)?;
        self.cursor += res as u64;
        Ok(res)
    }
}


struct KVDiskWriter<W: Write> {
    writer: BufWriter<W>,
    cursor: u64
}

impl<W: Write+Seek> KVDiskWriter<W> {
    pub fn new(inner: W) -> Result<Self> {
        let writer = BufWriter::new(inner);
        Ok(
            Self {
            writer: writer,
            cursor: 0,
        })
    }
    
    pub fn write_entry(&mut self, serialized: String) -> Result<(u64, u64)> {
        let len = self.writer.write(serialized.as_bytes())? as u64;
        let old_pos = self.cursor;
        self.cursor = old_pos + len;
        self.writer.flush()?;
        Ok((old_pos, len))
    }
}

impl<W: Write+Seek> Write for KVDiskWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.writer.write(buf)?;
        self.cursor += res as u64;
        Ok(res) 
    }

    fn flush(&mut self) -> io::Result<()> { 
        self.writer.flush()
    }
}

