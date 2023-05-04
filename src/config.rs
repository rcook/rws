// The MIT License (MIT)
//
// Copyright (c) 2020-3 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use anyhow::{bail, Result};
use joatmon::read_text_file;
use std::path::Path;
use yaml_rust::yaml::{Array, Hash, Yaml};
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub struct ConfigObject {
    pub yaml: Yaml,
}

impl ConfigObject {
    pub fn read_config_file(path: &Path) -> Result<Option<Self>> {
        let yaml = read_text_file(path)?;
        let mut docs = YamlLoader::load_from_str(&yaml)?;
        match docs.len() {
            0 => Ok(None),
            1 => Ok(Some(Self::new(docs.remove(0)))),
            _ => bail!("Invalid workspace config file {}", path.display()),
        }
    }

    pub fn into_bool(self) -> Option<bool> {
        self.yaml.into_bool()
    }

    pub fn as_str(&self) -> Option<&str> {
        self.yaml.as_str()
    }

    pub fn into_string(self) -> Option<String> {
        self.yaml.into_string()
    }

    pub fn into_hash(self) -> Option<ConfigHash> {
        self.yaml.into_hash().map(ConfigHash::new)
    }

    pub fn into_vec(self) -> Option<ConfigArray> {
        self.yaml.into_vec().map(ConfigArray::new)
    }

    fn new(yaml: Yaml) -> Self {
        Self { yaml }
    }
}

#[derive(Debug)]
pub struct ConfigHash {
    hash: Hash,
}

impl ConfigHash {
    pub fn get(&self, key: &str) -> Option<ConfigObject> {
        self.hash
            .get(&Yaml::String(key.to_string()))
            .map(|x| ConfigObject::new(x.clone()))
    }

    pub fn keys(&self) -> Option<Vec<String>> {
        let mut keys = Vec::new();
        for k in self.hash.keys() {
            if let Some(s) = k.as_str() {
                keys.push(s.to_string())
            } else {
                return None;
            }
        }
        Some(keys)
    }

    fn new(hash: Hash) -> Self {
        ConfigHash { hash }
    }
}

pub struct ConfigArray {
    array: Array,
}

impl ConfigArray {
    fn new(array: Array) -> ConfigArray {
        ConfigArray { array }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn get(&self, index: usize) -> ConfigObject {
        ConfigObject::new(self.array[index].clone())
    }
}
