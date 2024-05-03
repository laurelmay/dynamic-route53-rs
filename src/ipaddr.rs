use crate::config::{DnsProtocol, DnsServerConfig};
use crate::errors::AddressResolutionError;
use hickory_client::client::{AsyncClient, ClientHandle, Signer};
use hickory_client::error::ClientError;
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::proto::tcp::TcpClientConnect;
use hickory_client::proto::udp::UdpClientConnect;
use hickory_client::proto::xfer::{BufDnsStreamHandle, DnsExchangeBackground, DnsMultiplexer};
use hickory_client::proto::TokioTime;
use hickory_client::rr::{DNSClass, Name, RData, RecordType};
use hickory_client::tcp::TcpClientStream;
use hickory_client::udp::UdpClientStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::net::UdpSocket;

type AsyncTcpStream = AsyncIoTokioAsStd<TokioTcpStream>;
type AsyncDnsBackground<S> = DnsExchangeBackground<S, TokioTime>;
type AsyncTcpDnsBackground =
    AsyncDnsBackground<DnsMultiplexer<TcpClientStream<AsyncTcpStream>, Signer>>;
type AsyncUdpDnsBackground = AsyncDnsBackground<UdpClientStream<UdpSocket>>;

enum ConnectionWrapper {
    Tcp((TcpClientConnect<AsyncTcpStream>, BufDnsStreamHandle)),
    Udp(UdpClientConnect<UdpSocket>),
}

pub enum ClientWrapper {
    Tcp((AsyncClient, AsyncTcpDnsBackground)),
    Udp((AsyncClient, AsyncUdpDnsBackground)),
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
        DnsProtocol::UDP => ConnectionWrapper::Udp(UdpClientStream::new(address)),
        DnsProtocol::TCP => ConnectionWrapper::Tcp(TcpClientStream::new(address)),
    })
}

pub async fn create_dns_client(
    server: &DnsServerConfig,
) -> Result<ClientWrapper, AddressResolutionError> {
    let address = format!("{}:{}", server.host, server.port).parse()?;
    let conn = create_connection(&server.protocol, address)?;
    let client = match conn {
        ConnectionWrapper::Tcp((stream, sender)) => {
            ClientWrapper::Tcp(AsyncClient::new(stream, sender, None).await?)
        }
        ConnectionWrapper::Udp(conn) => ClientWrapper::Udp(AsyncClient::connect(conn).await?),
    };

    Ok(client)
}

async fn current_record_value(
    host: &str,
    client: &mut AsyncClient,
) -> Result<Vec<IpAddr>, AddressResolutionError> {
    let name = Name::from_str(host)?;
    let response = client.query(name, DNSClass::IN, RecordType::A).await?;
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

pub async fn is_current_address(
    host: &str,
    client: &mut AsyncClient,
    ip: &Ipv4Addr,
) -> Result<bool, AddressResolutionError> {
    println!("Comparing {} to {}", host, ip);
    let current_value = current_record_value(host, client).await;
    println!("Lookup result: {:?}", current_value);
    match current_value {
        Err(AddressResolutionError::DnsResolutionFailure(_)) => Ok(false),
        Err(e) => Err(e),
        Ok(resolved_ips) if resolved_ips.is_empty() => Ok(false),
        Ok(resolved_ips) => Ok(resolved_ips.iter().any(|addr| *addr == *ip)),
    }
}
