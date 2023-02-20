use sled::Db;

use crate::{
    KvsEngine, KVError,
    error::Result
};
use std::{
    path::PathBuf,
    str,
};

#[derive(Clone)]
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
    fn set(&self, key: String, value: String) -> Result<()> {
        let tree: &Db = &self.db;
        tree.insert(key.as_bytes(), value.as_bytes())?;
        tree.flush()?;
        Ok(())
    }

    /// Gets the string value of a given string key.
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        let tree: &Db = &self.db;
        if let Some(res) = tree.get(key.as_bytes())? {
            let s = String::from(str::from_utf8(&res)?);
            return Ok(Some(s))
        }
        Ok(None)
    }

    /// Removes a given key.
    /// It returns `KeyNotFound` if the given key is not found.
    fn remove(&self, key: String) -> Result<()> {
        let tree: &Db = &self.db;
        tree.remove(key.as_bytes())?.ok_or(KVError::KeyNoExist)?;
        tree.flush()?;
        Ok(())
    }
}