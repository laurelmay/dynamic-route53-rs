use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DnsProtocol {
    TCP,
    #[default]
    UDP,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsServerConfig {
    pub host: String,
    #[serde(default = "default_dns_port")]
    pub port: u16,
    #[serde(default)]
    pub protocol: DnsProtocol,
}

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
    #[serde(default = "default_dns_server")]
    pub dns_server: DnsServerConfig,
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

fn default_dns_port() -> u16 {
    53
}

fn default_dns_server() -> DnsServerConfig {
    DnsServerConfig {
        host: "1.1.1.1".to_string(),
        port: 53,
        protocol: DnsProtocol::UDP,
    }
}

pub fn parse_config(config_file: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    Ok(serde_yaml::from_reader(File::open(config_file)?)?)
}
