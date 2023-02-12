use crate::error::{Result};
use std::path::PathBuf;
use crate::KvsEngine;

pub struct SledKvsEngine {
    
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        todo!()
    }
}


impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        Ok(())
    }

    /// Gets the string value of a given string key.
    /// Returns `None` if the given key does not exist.
    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(None)
    }

    /// Removes a given key.
    /// It returns `KeyNotFound` if the given key is not found.
    fn remove(&mut self, key: String) -> Result<()> {
        Ok(())
    }
}