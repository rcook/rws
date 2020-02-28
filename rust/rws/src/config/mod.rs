use std::path::Path;
use yaml_rust::{ YamlLoader };
use yaml_rust::yaml::{ Hash, Yaml };

pub struct Config {
    doc: Yaml
}

impl Config {
    pub fn read_yaml_file(path: &Path) -> std::io::Result<Config> {
        let yaml = std::fs::read_to_string(path)?;
        let mut docs = YamlLoader::load_from_str(&yaml).unwrap();
        if docs.len() != 1 {
            panic!("Invalid workspace config file");
        }

        Ok(Config { doc: docs.remove(0) })
    }

    pub fn as_hash(&self) -> Option<ConfigHash> {
        let h = self.doc.as_hash()?;
        Some(ConfigHash::new(h))
    }
}

pub struct ConfigHash<'a> {
    hash: &'a Hash
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

    fn new(hash: &Hash) -> ConfigHash {
        ConfigHash { hash: hash }
    }

    fn get_item(&self, key: &str) -> Option<&'a Yaml> {
        self.hash.get(&Yaml::String(key.to_string()))
    }
}
