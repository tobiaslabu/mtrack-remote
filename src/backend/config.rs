use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use dioxus::logger::tracing::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config path does not exist!")]
    PathDoesNotExist,
    #[error("Could not read file! {0}")]
    CouldNotReadFile(String),
    #[error("Could not get config directory!")]
    CouldNotGetConfigDir,
    #[error("Could not open file! {0}")]
    CouldNotOpenFile(String),
    #[error("Could not deserialize config! {0}")]
    CouldNotDeserialize(String),
    #[error("Could not serialize config! {0}")]
    CouldNotSerialize(String),
    #[error("Could not write config file! {0}")]
    CouldNotWriteFile(String),
    #[error("Could not create config directory! {0}")]
    CouldNotCreateDirectory(String),
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    pub mtrack_addr: SocketAddr,
    pub listen_port: u16,
}

pub const DEFAULT_MTRACK_PORT: u16 = 43234;
pub const DEFAULT_LISTEN_PORT: u16 = 43236;

impl Config {
    pub fn new() -> Self {
        Self {
            mtrack_addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), DEFAULT_MTRACK_PORT),
            listen_port: DEFAULT_LISTEN_PORT,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mtrack_addr: SocketAddr::new(
                std::net::IpAddr::V4(Ipv4Addr::LOCALHOST),
                DEFAULT_MTRACK_PORT,
            ),
            listen_port: DEFAULT_LISTEN_PORT,
        }
    }
}

fn get_config_dir() -> Result<PathBuf, ConfigError> {
    let dir = match dirs::config_local_dir() {
        Some(dir) => dir,
        None => return Err(ConfigError::CouldNotGetConfigDir),
    };
    Ok(dir.join("mtrack-remote"))
}

fn get_config_file_path() -> Result<PathBuf, ConfigError> {
    let dir = get_config_dir()?;
    Ok(dir.join("config.json"))
}

impl Config {
    pub fn read_config() -> Result<Config, ConfigError> {
        let config_file_path = get_config_file_path()?;
        if !config_file_path.exists() {
            return Ok(Config::default());
        }

        let file = match File::open(config_file_path) {
            Ok(file) => file,
            Err(err) => return Err(ConfigError::CouldNotOpenFile(err.to_string())),
        };
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        let serialized = match reader.read_to_string(&mut buf) {
            Ok(bytes_read) => {
                debug!("Read {bytes_read} bytes of config file.");
                buf
            }
            Err(err) => return Err(ConfigError::CouldNotReadFile(err.to_string())),
        };

        match serde_json::from_str::<Config>(&serialized) {
            Ok(config) => Ok(config),
            Err(err) => Err(ConfigError::CouldNotDeserialize(err.to_string())),
        }
    }

    pub fn write_config(&self) -> Result<(), ConfigError> {
        let serialized = match serde_json::to_string(&self) {
            Ok(serialized) => serialized,
            Err(err) => return Err(ConfigError::CouldNotSerialize(err.to_string())),
        };

        let config_file_path = get_config_file_path()?;
        if !config_file_path.exists() {
            if let Some(path) = config_file_path.parent() {
                match std::fs::create_dir_all(path) {
                    Ok(_ok) => debug!("Created config path."),
                    Err(err) => return Err(ConfigError::CouldNotCreateDirectory(err.to_string())),
                };
            } else {
                return Err(ConfigError::PathDoesNotExist);
            }
        }

        let mut file = match File::create(config_file_path) {
            Ok(file) => file,
            Err(err) => return Err(ConfigError::CouldNotOpenFile(err.to_string())),
        };

        match file.write_all(serialized.as_bytes()) {
            Ok(_result) => Ok(()),
            Err(err) => Err(ConfigError::CouldNotWriteFile(err.to_string())),
        }
    }
}
