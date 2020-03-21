use crate::config::ConfigObject;

pub struct Variables {
    pub values: Vec<(String, ConfigObject)>,
}

impl Variables {
    pub fn new(values: Vec<(String, ConfigObject)>) -> Self {
        Self { values }
    }
}
