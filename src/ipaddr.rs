use crate::errors::AddressResolutionError;
use std::net::{IpAddr, ToSocketAddrs};

pub async fn get_ip(hostname: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(hostname).await?.text().await?;
    Ok(resp)
}

fn current_record_value(host: &str) -> Result<IpAddr, AddressResolutionError> {
    // ToSocketAddrs requires a properly-formed socket which requires a port to
    // be present. We never actually use the port for anything else but it has to
    // be there.
    let host_socket = format!("{}:443", host);
    let host_ip = host_socket.to_socket_addrs()?.next();
    if let Some(host_ip) = host_ip {
        return Ok(host_ip.ip());
    }

    Err(AddressResolutionError::DnsResolutionFailure(
        host.to_string(),
    ))
}

pub fn is_current_address(host: &str, ip: &str) -> Result<bool, AddressResolutionError> {
    println!("Comparing {} to {}", host, ip);
    match current_record_value(host) {
        Err(AddressResolutionError::DnsResolutionFailure(_)) => Ok(false),
        Err(e) => Err(e),
        Ok(resolved_ip) => Ok(resolved_ip == ip.parse::<IpAddr>()?),
    }
}
