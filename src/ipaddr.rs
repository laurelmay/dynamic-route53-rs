use crate::config::{DnsProtocol, DnsServerConfig};
use crate::errors::AddressResolutionError;
use hickory_client::client::{Client, SyncClient};
use hickory_client::error::ClientError;
use hickory_client::rr::{DNSClass, Name, RData, RecordType};
use hickory_client::tcp::TcpClientConnection;
use hickory_client::udp::UdpClientConnection;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

enum ConnectionWrapper {
    Tcp(TcpClientConnection),
    Udp(UdpClientConnection),
}

pub enum ClientWrapper {
    Tcp(SyncClient<TcpClientConnection>),
    Udp(SyncClient<UdpClientConnection>),
}

pub async fn get_ip(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}

fn create_connection(
    protocol: &DnsProtocol,
    address: SocketAddr,
) -> Result<ConnectionWrapper, ClientError> {
    Ok(match protocol {
        DnsProtocol::UDP => ConnectionWrapper::Udp(UdpClientConnection::new(address)?),
        DnsProtocol::TCP => ConnectionWrapper::Tcp(TcpClientConnection::new(address)?),
    })
}

pub fn create_dns_client(
    server: &DnsServerConfig,
) -> Result<ClientWrapper, AddressResolutionError> {
    let address = format!("{}:{}", server.host, server.port).parse()?;
    let conn = create_connection(&server.protocol, address)?;
    let client = match conn {
        ConnectionWrapper::Tcp(conn) => ClientWrapper::Tcp(SyncClient::new(conn)),
        ConnectionWrapper::Udp(conn) => ClientWrapper::Udp(SyncClient::new(conn)),
    };

    Ok(client)
}

fn current_record_value(
    host: &str,
    client: ClientWrapper,
) -> Result<Vec<IpAddr>, AddressResolutionError> {
    let name = Name::from_str(host)?;
    let response = match client {
        ClientWrapper::Tcp(wrapped) => wrapped.query(&name, DNSClass::IN, RecordType::A)?,
        ClientWrapper::Udp(wrapped) => wrapped.query(&name, DNSClass::IN, RecordType::A)?,
    };
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
    client: ClientWrapper,
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
