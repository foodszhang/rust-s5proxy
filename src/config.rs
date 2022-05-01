use serde_derive::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub ip: Option<String>,
    pub port: Option<u16>,
}

impl Config {
    pub fn from_file(filename: &Path) -> Result<Config, io::Error> {
        let content = fs::read_to_string(filename)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
