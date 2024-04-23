use crate::config::DnsServerConfig;
use crate::errors::AddressResolutionError;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientConnection;

pub async fn get_ip(hostname: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(hostname).await?.text().await?;
    Ok(resp)
}

pub fn create_dns_client(server: &DnsServerConfig) -> Result<impl Client, AddressResolutionError> {
    let address = format!("{}:{}", server.host, server.port).parse()?;
    let conn = UdpClientConnection::new(address)?;
    let client = SyncClient::new(conn);

    Ok(client)
}

fn current_record_value(
    host: &str,
    client: impl Client,
) -> Result<Vec<IpAddr>, AddressResolutionError> {
    let name = Name::from_str(host)?;
    let response = client.query(&name, DNSClass::IN, RecordType::A)?;
    let answers = response.answers();

    let mut results = vec![];
    for answer in answers {
        if let Some(RData::A(ref ip)) = answer.data() {
            results.push(IpAddr::V4(ip.0));
        }
    }
    let results = results;

    Ok(results)
}

pub fn is_current_address(
    host: &str,
    client: impl Client,
    ip: &Ipv4Addr,
) -> Result<bool, AddressResolutionError> {
    println!("Comparing {} to {}", host, ip);
    match current_record_value(host, client) {
        Err(AddressResolutionError::DnsResolutionFailure(_)) => Ok(false),
        Err(e) => Err(e),
        Ok(resolved_ips) if resolved_ips.is_empty() => Ok(false),
        Ok(resolved_ips) => Ok(resolved_ips.iter().any(|addr| *addr == *ip)),
    }
}
