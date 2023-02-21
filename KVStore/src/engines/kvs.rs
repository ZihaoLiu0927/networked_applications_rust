use crate::error::{KVError, Result};
use crate::logfile;
use crate::KvsEngine;

use crossbeam_skiplist::SkipMap;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::sync::atomic::AtomicU64;
use std::sync::{atomic::Ordering, Arc, Mutex};

use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1MB

#[derive(Clone)]
pub struct KvStore {
    indexmap: Arc<SkipMap<String, DiskPos>>,
    reader: KvStoreReader,
    writer: Arc<Mutex<KvStoreWriter>>,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;

        let indexmap: Arc<SkipMap<String, DiskPos>> = Arc::new(SkipMap::new());

        let gen_list = collect_file_identifiers(&path)?;

        let mut readers = HashMap::new();

        let mut need_compact = 0;
        for &gen in &gen_list {
            let mut reader = KVDiskReader::new(File::open(logfile!(path, gen))?)?;
            need_compact += reader.load_log_from_disk(&indexmap, gen)?;
            readers.insert(gen, reader);
        }

        let curr_gen = gen_list.last().unwrap_or(&0) + 1;

        let writer = create_new_log(&path, curr_gen)?;

        let reader = KvStoreReader {
            path: Arc::clone(&path),
            curr_compact: Arc::new(AtomicU64::new(0)),
            readers: RefCell::new(readers),
        };

        let writer = Arc::new(Mutex::new(KvStoreWriter {
            path: Arc::clone(&path),
            indexmap: Arc::clone(&indexmap),
            reader: reader.clone(),
            curr_gen,
            need_compact,
            writer,
        }));

        Ok(Self {
            indexmap: indexmap,
            reader,
            writer,
        })
    }
}

impl KvsEngine for KvStore {
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }

    fn set(&self, key: String, val: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, val)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(pos) = self.indexmap.get(&key) {
            if let Command::Set { value, .. } = self.reader.read_command(pos.value())? {
                Ok(Some(value))
            } else {
                Err(KVError::GetMethodError)
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

impl From<(u64, u64, u64)> for DiskPos {
    fn from((gen, pos, len): (u64, u64, u64)) -> Self {
        Self { gen, pos, len }
    }
}

struct KvStoreReader {
    path: Arc<PathBuf>,
    readers: RefCell<HashMap<u64, KVDiskReader<File>>>,
    curr_compact: Arc<AtomicU64>,
}

impl KvStoreReader {
    fn close_stale_read_handles(&self) {
        let curr_compact = self.curr_compact.load(Ordering::SeqCst);
        let mut readers = self.readers.borrow_mut();

        let delete_files: Vec<u64> = readers
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| *key < curr_compact)
            .collect();

        for num in delete_files {
            readers.remove(&num);
        }
    }

    fn read_entry_then<R, F>(&self, pos: &DiskPos, then: F) -> Result<R>
    where
        F: FnOnce(io::Take<&mut KVDiskReader<File>>) -> Result<R>,
    {
        self.close_stale_read_handles();

        let mut readers = self.readers.borrow_mut();

        if !readers.contains_key(&pos.gen) {
            let r = KVDiskReader::new(File::open(logfile!(self.path, pos.gen))?)?;
            readers.insert(pos.gen, r);
        }

        let r = readers.get_mut(&pos.gen).unwrap();
        r.seek(SeekFrom::Start(pos.pos))?;
        let cmd_reader = r.take(pos.len);

        then(cmd_reader)
    }

    fn read_command(&self, pos: &DiskPos) -> Result<Command> {
        self.read_entry_then(pos, |take| Ok(serde_json::from_reader(take)?))
    }
}

impl Clone for KvStoreReader {
    fn clone(&self) -> Self {
        Self {
            path: Arc::clone(&self.path),
            curr_compact: Arc::clone(&self.curr_compact),
            readers: RefCell::new(HashMap::new()),
        }
    }
}

struct KVDiskReader<R: Read + Seek> {
    reader: BufReader<R>,
    cursor: u64,
}

impl<R: Read + Seek> Seek for KVDiskReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.cursor = self.reader.seek(pos)?;
        Ok(self.cursor)
    }
}

impl<R: Read + Seek> Read for KVDiskReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.reader.read(buf)?;
        self.cursor += res as u64;
        Ok(res)
    }
}

impl<R: Read + Seek> KVDiskReader<R> {
    pub fn new(inner: R) -> Result<Self> {
        let reader = BufReader::new(inner);
        Ok(Self {
            reader: reader,
            cursor: 0,
        })
    }

    pub fn load_log_from_disk(
        &mut self,
        map: &Arc<SkipMap<String, DiskPos>>,
        fgen: u64,
    ) -> Result<u64> {
        let ref_reader = self.reader.get_mut();
        ref_reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(ref_reader).into_iter::<Command>();
        let mut pos;
        let mut need_compact = 0;
        loop {
            pos = stream.byte_offset() as u64;
            if let Some(entry) = stream.next() {
                let len = stream.byte_offset() as u64 - pos;
                match entry? {
                    Command::Set { key: k, value: _v } => {
                        if let Some(old_entry) = map.get(&k) {
                            need_compact += old_entry.value().len;
                        }
                        map.insert(
                            k,
                            DiskPos {
                                gen: fgen,
                                pos,
                                len,
                            },
                        );
                    }
                    Command::Remove { key: k } => {
                        if let Some(old_entry) = map.remove(&k) {
                            need_compact += old_entry.value().len;
                        }
                        need_compact += len;
                    }
                }
                continue;
            }
            return Ok(need_compact);
        }
    }
}

struct KvStoreWriter {
    // copy from KvStore
    path: Arc<PathBuf>,
    indexmap: Arc<SkipMap<String, DiskPos>>,
    reader: KvStoreReader,
    // writer fields
    writer: KVDiskWriter<File>,
    curr_gen: u64,
    need_compact: u64,
}

impl KvStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            value: value,
        };
        let serialized = serde_json::to_string_pretty(&command)?;
        let (pos, len) = self.writer.write_entry(serialized)?;
        self.writer.flush()?;

        let diskpos = DiskPos {
            gen: self.curr_gen,
            pos,
            len,
        };

        if let Some(entry) = self.indexmap.get(&key) {
            self.need_compact += entry.value().len;
        }

        self.indexmap.insert(key, diskpos);

        if self.need_compact > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.indexmap.contains_key(&key) {
            let command = Command::Remove { key: key.clone() };
            let serialized = serde_json::to_string_pretty(&command)?;
            let (_pos, len) = self.writer.write_entry(serialized)?;
            self.writer.flush()?;

            let stale_len = self
                .indexmap
                .remove(&key)
                .expect("Key not found!")
                .value()
                .len;

            self.need_compact += stale_len;
            // the "remove" command itself can be deleted in the next compaction
            self.need_compact += len;

            if self.need_compact > COMPACTION_THRESHOLD {
                self.compact()?;
            }

            Ok(())
        } else {
            Err(KVError::KeyNoExist)
        }
    }

    fn compact(&mut self) -> Result<()> {
        let gen_compact = self.curr_gen + 1;
        self.curr_gen += 2;

        self.writer = create_new_log(&self.path, self.curr_gen)?;

        let mut compact_writer = create_new_log(&self.path, gen_compact)?;

        let mut pos = 0;
        for entry in self.indexmap.iter() {
            let len = self.reader.read_entry_then(&entry.value(), |mut take| {
                Ok(io::copy(&mut take, &mut compact_writer)?)
            })?;

            self.indexmap
                .insert(entry.key().clone(), (gen_compact, pos, len).into());

            pos += len;
        }

        compact_writer.flush()?;

        self.reader
            .curr_compact
            .store(gen_compact, Ordering::SeqCst);
        self.reader.close_stale_read_handles();

        let old_files: Vec<u64> = collect_file_identifiers(&self.path)?
            .into_iter()
            .filter(|&gen| gen < gen_compact)
            .collect();

        for gen in old_files {
            fs::remove_file(logfile!(self.path, gen))?;
        }

        self.need_compact = 0;

        Ok(())
    }
}

struct KVDiskWriter<W: Write> {
    writer: BufWriter<W>,
    cursor: u64,
}

impl<W: Write + Seek> Write for KVDiskWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.writer.write(buf)?;
        self.cursor += res as u64;
        Ok(res)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> KVDiskWriter<W> {
    pub fn new(inner: W) -> Result<Self> {
        let writer = BufWriter::new(inner);
        Ok(Self {
            writer: writer,
            cursor: 0,
        })
    }

    pub fn write_entry(&mut self, serialized: String) -> Result<(u64, u64)> {
        let len = self.writer.write(serialized.as_bytes())? as u64;
        let old_pos = self.cursor;
        self.cursor = old_pos + len;
        Ok((old_pos, len))
    }
}

fn create_new_log(path: &Path, curr_gen: u64) -> Result<KVDiskWriter<File>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(logfile!(path, curr_gen))?;

    let writer = KVDiskWriter::new(file)?;
    Ok(writer)
}
