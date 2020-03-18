use crate::error::{user_error_result, AppError, Result};

use rlua::{Context, Value};
use std::path::Path;
use yaml_rust::yaml::{Array, Hash, Yaml};
use yaml_rust::{ScanError, YamlLoader};

pub struct ConfigObject {
    yaml: Yaml,
}

impl std::convert::From<ScanError> for AppError {
    fn from(error: ScanError) -> Self {
        AppError::System("Yaml", error.to_string())
    }
}

impl ConfigObject {
    pub fn read_config_file(path: &Path) -> Result<Option<Self>> {
        let yaml = std::fs::read_to_string(path)?;
        let mut docs = YamlLoader::load_from_str(&yaml)?;
        match docs.len() {
            0 => Ok(None),
            1 => Ok(Some(Self::new(docs.remove(0)))),
            _ => user_error_result(format!("Invalid workspace config file {}", path.display())),
        }
    }

    pub fn into_bool(self) -> Option<bool> {
        self.yaml.into_bool()
    }

    pub fn into_string(self) -> Option<String> {
        self.yaml.into_string()
    }

    pub fn into_hash(self) -> Option<ConfigHash> {
        self.yaml.into_hash().map(|x| ConfigHash::new(x))
    }

    pub fn into_vec(self) -> Option<ConfigArray> {
        self.yaml.into_vec().map(|x| ConfigArray::new(x))
    }

    fn new(yaml: Yaml) -> Self {
        Self { yaml: yaml }
    }

    pub fn to_lua<'a>(&self, lua_ctx: Context<'a>) -> Result<Value<'a>> {
        Ok(Self::translate(lua_ctx, &self.yaml)?)
    }

    fn translate<'a>(lua_ctx: Context<'a>, yaml: &Yaml) -> rlua::Result<Value<'a>> {
        match yaml {
            Yaml::String(value) => lua_ctx.create_string(&value).map(|str| Value::String(str)),
            Yaml::Array(value) => lua_ctx
                .create_sequence_from(
                    value
                        .iter()
                        .map(|x| Self::translate(lua_ctx, x))
                        .collect::<rlua::Result<Vec<_>>>()?,
                )
                .map(|seq| Value::Table(seq)),
            Yaml::Hash(value) => lua_ctx
                .create_table_from(
                    value
                        .iter()
                        .map(|(k, v)| {
                            k.as_str()
                                .ok_or_else(|| {
                                    rlua::Error::RuntimeError(String::from("Invalid YAML"))
                                })
                                .and_then(|k_str| {
                                    lua_ctx.create_string(k_str).and_then(|key| {
                                        Self::translate(lua_ctx, v)
                                            .map(|value| (Value::String(key), value))
                                    })
                                })
                        })
                        .collect::<rlua::Result<Vec<(Value, Value)>>>()?,
                )
                .map(|table| Value::Table(table)),
            _ => panic!("NotImpl2"),
        }
    }
}

pub struct ConfigHash {
    hash: Hash,
}

impl ConfigHash {
    pub fn get(&self, key: &str) -> Option<ConfigObject> {
        self.hash
            .get(&Yaml::String(key.to_string()))
            .map(|x| ConfigObject::new(x.clone()))
    }

    pub fn keys(&self) -> Vec<String> {
        self.hash
            .keys()
            .map(|x| x.as_str().expect("Keys must be strings").to_string())
            .collect()
    }

    fn new(hash: Hash) -> Self {
        ConfigHash { hash: hash }
    }
}

pub struct ConfigArray {
    array: Array,
}

impl ConfigArray {
    fn new(array: Array) -> ConfigArray {
        ConfigArray { array: array }
    }

    pub fn len(&self) -> usize {
        self.array.len()
    }

    pub fn get(&self, index: usize) -> ConfigObject {
        ConfigObject::new(self.array[index].clone())
    }
}
