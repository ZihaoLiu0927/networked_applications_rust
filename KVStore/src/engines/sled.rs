use crate::error::{Result};
use std::path::{PathBuf};
use std::str;
use crate::{KvsEngine, KVError};

use sled::Db;

pub struct SledKvsEngine {
    db: Db
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        let db = sled::open(path.into())?;

        Ok(Self {
            db
        })
    }
}


impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    /// Gets the string value of a given string key.
    /// Returns `None` if the given key does not exist.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(res) = self.db.get(key.as_bytes())? {
            let s = String::from(str::from_utf8(&res)?);
            return Ok(Some(s))
        }
        Ok(None)
    }

    /// Removes a given key.
    /// It returns `KeyNotFound` if the given key is not found.
    fn remove(&mut self, key: String) -> Result<()> {
        self.db.remove(key.as_bytes())?.ok_or(KVError::KeyNoExist)?;
        self.db.flush()?;
        Ok(())
    }
}