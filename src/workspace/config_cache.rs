use crate::config::Config;
use crate::error::Result;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load and cache workspace configurations
pub struct ConfigCache {
    configs: HashMap<PathBuf, Option<Config>>,
}

impl ConfigCache {
    /// Constructor
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Read workspace configuration from specified file and cache result
    pub fn get_config(&mut self, path: &Path) -> Result<Option<&Config>> {
        Ok(match self.configs.entry(path.to_path_buf()) {
            Entry::Vacant(entry) => entry.insert(Config::read_config_file(path)?),
            Entry::Occupied(entry) => entry.into_mut(),
        }
        .as_ref())
    }
}
