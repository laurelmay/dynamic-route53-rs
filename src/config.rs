use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub hosted_zone_id: String,
    pub record_name: String,
    #[serde(default = "default_ttl")]
    pub ttl: i64,
    #[serde(default = "default_ip_service")]
    pub ip_check: String,
    #[serde(default = "default_force_set")]
    pub always_update_record: bool,
}

fn default_ip_service() -> String {
    "https://checkip.amazonaws.com".to_string()
}

fn default_ttl() -> i64 {
    300
}

fn default_force_set() -> bool {
    false
}

pub fn parse_config(config_file: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    Ok(serde_yaml::from_reader(File::open(config_file)?)?)
}
