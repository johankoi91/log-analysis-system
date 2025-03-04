use std::env;
use serde::{Deserialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub(crate) connect_ips: ConnectIps,
}

#[derive(Debug, Deserialize)]
pub struct ConnectIps {
    pub(crate) elasticsearch: String,
    pub(crate) log_source_edges: Vec<String>,
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file_path = env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "/Users/hanxiaoqing/log-searching/logs_filter/config/config.yaml".to_string());
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
