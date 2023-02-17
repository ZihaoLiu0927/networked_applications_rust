use crate::KvsEngine;
use crate::error::{KVError, Result};
use crate::logfile;

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    sync::{Arc, Mutex}
};


use std::{
    ffi::OsStr,
    collections::HashMap,
    path::{PathBuf, Path},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write, SeekFrom, Seek, BufReader, BufWriter},
};


const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1MB
const UNABLE_HOLD_LOCK: &str = "unable to hold lock.";


#[derive(Clone)]
pub struct KvStore {
    path: PathBuf,
    indexmap: Arc<Mutex<HashMap<String, DiskPos>>>,
    readers: Arc<Mutex<HashMap<u64, KVDiskReader<File>>>>,
    writer: Arc<Mutex<KVDiskWriter<File>>>,
    curr_gen: u64,
    need_compact: Arc<Mutex<u64>>,
}

impl KvStore {

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut indexmap: Arc<Mutex<HashMap<String, DiskPos>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut need_compact = 0;

        let readers = Arc::new(Mutex::new(HashMap::new())); 
        let gen_list = collect_file_identifiers(&path)?;

        for &gen in &gen_list {
            let mut reader = KVDiskReader::new(File::open(logfile!(path, gen))?)?;
            need_compact += reader.load_log_from_disk(&mut indexmap, gen)?;
            readers
                .lock()
                .expect(UNABLE_HOLD_LOCK)
                .insert(gen, reader);
        }

        let curr_gen = gen_list
            .last()
            .unwrap_or(&0) + 1;

        let writer = Arc::new(Mutex::new(
            Self::create_new_log(&path, curr_gen, &readers)?
        ));

        let need_compact = Arc::new(Mutex::new(need_compact));

        Ok(KvStore {
            path,
            indexmap,
            readers,
            writer,
            curr_gen,
            need_compact,
        })

    }

    fn create_new_log(path: &Path, curr_gen: u64, readers: &Arc<Mutex<HashMap<u64, KVDiskReader<File>>>>) -> Result<KVDiskWriter<File>> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(logfile!(path, curr_gen))?;

        let writer = KVDiskWriter::new(file.try_clone()?)?;
        readers
            .lock()
            .expect(UNABLE_HOLD_LOCK)
            .insert(curr_gen, KVDiskReader::new(file)?);
        
        Ok(writer)
    }

    fn compaction(&mut self) -> Result<()> {

        let gen_compact = self.curr_gen + 1;
        let mut writer = Self::create_new_log(&self.path, gen_compact, &self.readers)?;

        let mut pos = 0;
        for entry in &mut self.indexmap.lock().expect(UNABLE_HOLD_LOCK).values_mut() {
            let mut reader_locked = self.readers
                .lock()
                .expect(UNABLE_HOLD_LOCK);

            let reader = reader_locked.get_mut(&entry.gen).expect(UNABLE_HOLD_LOCK);

            if reader.cursor != entry.pos {
                reader.seek(SeekFrom::Start(entry.pos))?;
            }
            let mut read_from = reader.take(entry.len);
            let len = io::copy(&mut read_from, &mut writer)?;
            *entry = (gen_compact, pos, len).into();
            pos += len;
        }


        let gen_newlog = self.curr_gen + 2;
        self.writer = Arc::new(Mutex::new(
            Self::create_new_log(&self.path, gen_newlog, &self.readers)?
        ));

        let old_files: Vec<u64> = self.readers
                .lock()
                .expect(UNABLE_HOLD_LOCK)
                .keys()
                .filter(|&gen| *gen <= self.curr_gen)
                .map(|x| *x)
                .collect();
        
        for gen in old_files {
            fs::remove_file(logfile!(self.path, gen))?;
            self.readers.
                lock().
                expect(UNABLE_HOLD_LOCK)
                .remove(&gen);
        }

        self.curr_gen = gen_newlog;
        *self.need_compact.lock().expect(UNABLE_HOLD_LOCK) = 0;
        
        Ok(())
    }

}

impl KvsEngine for KvStore {
    fn remove(&self, key: String) -> Result<()> {
        let check = self.get(key.clone())?;
        if check.is_none() {
            return Err(KVError::KeyNoExist)
        }

        if let Some(_) =  self.indexmap.lock().expect(UNABLE_HOLD_LOCK).get(&key) {
            let command = Command::Remove { key: key.clone() };
            let serialized = serde_json::to_string_pretty(&command)?;
            let (pos, len) = self.writer
                .lock()
                .expect(UNABLE_HOLD_LOCK)
                .write_entry(serialized)?;

            let diskpos = DiskPos {
                gen: self.curr_gen,
                pos,
                len,
            };
            self.indexmap
                .lock()
                .expect(UNABLE_HOLD_LOCK)
                .insert(key, diskpos);

        } else {
            return Err(KVError::KeyNoExist)
        }
        Ok(())
    }

    fn set(&self, key: String, val: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(), 
            value: val.clone(),
        };
        let serialized = serde_json::to_string_pretty(&command)?;
        let (pos, len) = self.writer
            .lock()
            .expect(UNABLE_HOLD_LOCK)
            .write_entry(serialized)?;

        let diskpos = DiskPos {
            gen: self.curr_gen,
            pos,
            len,
        };

        let stale_len = self.indexmap
            .lock()
            .expect(UNABLE_HOLD_LOCK)
            .insert(key, diskpos)
            .unwrap_or(DiskPos::new()).len;

        let mut size = self.need_compact.lock().expect(UNABLE_HOLD_LOCK);

        *size += stale_len;

        if *size > COMPACTION_THRESHOLD {
            self.compaction()?;
        }

        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(val) = self.indexmap
            .lock()
            .expect(UNABLE_HOLD_LOCK).get(&key) {
                let mut reader_locked = self.readers
                    .lock()
                    .expect(UNABLE_HOLD_LOCK);


                let reader = reader_locked.get_mut(&val.gen)
                    .expect("unable to get reference");
                
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

    pub fn load_log_from_disk(&mut self, map: &mut Arc<Mutex<HashMap<String, DiskPos>>>, fgen: u64) -> Result<u64> {
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
                            .lock()
                            .expect("unable to hold lock.")
                            .insert(k, DiskPos { gen: fgen, pos, len })
                            .unwrap_or(DiskPos::new());
                        need_compact += old_entry.len;
                    }
                    Command::Remove{key: k} => {
                        let old_entry = map
                            .lock()
                            .expect("unable to hold lock.")
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

