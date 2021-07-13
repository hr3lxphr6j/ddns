use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Service {
    pub name: String,
    pub config: HashMap<String, String>,
    pub domain: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub services: Vec<Service>,
}

impl AppConfig {
    pub fn new(file: impl AsRef<str>) -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.merge(File::with_name(file.as_ref()))?;
        s.try_into()
    }
}
