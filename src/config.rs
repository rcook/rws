use crate::error::{AppError, Result};
use std::path::Path;
use yaml_rust::yaml::{Array, Hash, Yaml};
use yaml_rust::{ScanError, YamlLoader};

pub struct Config {
    doc: Yaml,
}

impl std::convert::From<ScanError> for AppError {
    fn from(error: ScanError) -> Self {
        AppError::System("Yaml", error.to_string())
    }
}

impl Config {
    pub fn read_config_file(path: &Path) -> Result<Config> {
        let yaml = std::fs::read_to_string(path)?;
        let mut docs = YamlLoader::load_from_str(&yaml)?;
        if docs.len() != 1 {
            panic!("Invalid workspace config file");
        }

        Ok(Config {
            doc: docs.remove(0),
        })
    }

    pub fn as_hash(&self) -> Option<ConfigHash> {
        let h = self.doc.as_hash()?;
        Some(ConfigHash::new(h))
    }
}

pub struct ConfigHash<'a> {
    hash: &'a Hash,
}

impl<'a> ConfigHash<'a> {
    pub fn as_bool(&self, key: &str) -> Option<bool> {
        self.get_item(key)?.as_bool()
    }

    pub fn as_str(&self, key: &str) -> Option<&str> {
        self.get_item(key)?.as_str()
    }

    pub fn as_str_vec(&self, key: &str) -> Option<Vec<&str>> {
        self.get_item(key)?
            .as_vec()?
            .into_iter()
            .map(|x| x.as_str())
            .collect()
    }

    pub fn as_hash(&self, key: &str) -> Option<ConfigHash<'a>> {
        Some(ConfigHash::new(self.get_item(key)?.as_hash()?))
    }

    pub fn as_vec(&self, key: &str) -> Option<ConfigVec<'a>> {
        Some(ConfigVec::new(self.get_item(key)?.as_vec()?))
    }

    fn new(hash: &Hash) -> ConfigHash {
        ConfigHash { hash: hash }
    }

    fn get_item(&self, key: &str) -> Option<&'a Yaml> {
        self.hash.get(&Yaml::String(key.to_string()))
    }
}

pub struct ConfigVec<'a> {
    vec: &'a Array,
}

impl<'a> ConfigVec<'a> {
    fn new(vec: &Array) -> ConfigVec {
        ConfigVec { vec: vec }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn as_str(&self, index: usize) -> Option<&str> {
        self.vec[index].as_str()
    }
}
