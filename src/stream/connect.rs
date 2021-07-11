use super::{
    handshake::{Handshake, HandshakeError},
    Stream, StreamError,
};
use dbus_server_address_parser::{Address, Family, Tcp, Unix, UnixType};
use std::{
    io::Error as IoError,
    net::{IpAddr, SocketAddr},
};
use thiserror::Error;
use tokio::net::{lookup_host, TcpStream, UnixStream};

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Unix abstract is not yet supported")]
    UnixAbstractNotSupported,
    #[error("Nonce TCP is not yet supported")]
    NonceTcpNotSupported,
    #[error("Could not connect to any address")]
    CouldNotConnectToAnyAddress,
    #[error("Address is not connectable")]
    TcpAddressNotConnectable,
    #[error("Could not resolve IP addresses, which match the given IP family")]
    TcpResolveIpAddress,
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
    #[error("Handshake Error: {0}")]
    HandshakeError(#[from] HandshakeError),
}

impl Stream {
    async fn unix(unix: &Unix) -> Result<Stream, ConnectError> {
        match &unix.r#type {
            UnixType::Path(path) => {
                debug!("Connect to {}", path);
                let mut connection = UnixStream::connect(path).await?;
                Handshake::handshake(&mut connection, true).await?;
                Ok(Stream::Unix(connection))
            }
            UnixType::Abstract(_) => Err(ConnectError::UnixAbstractNotSupported),
            x => panic!("This should not happen: {}", x),
        }
    }

    fn tcp_family_match(socket_addr: &SocketAddr, family: &Option<Family>) -> bool {
        if let Some(family) = family {
            match family {
                Family::Ipv4 => socket_addr.is_ipv4(),
                Family::Ipv6 => socket_addr.is_ipv6(),
            }
        } else {
            true
        }
    }

    async fn tcp_connect(
        socket_addr: &SocketAddr,
        family: &Option<Family>,
    ) -> Result<TcpStream, ConnectError> {
        if !Stream::tcp_family_match(socket_addr, family) {
            return Err(ConnectError::TcpResolveIpAddress);
        }

        debug!("Connect to {}", socket_addr);
        let mut tcp_stream = TcpStream::connect(socket_addr).await?;
        Handshake::handshake(&mut tcp_stream, false).await?;
        Ok(tcp_stream)
    }

    async fn tcp(tcp: &Tcp) -> Result<Stream, ConnectError> {
        let host = tcp.host.as_ref().unwrap();
        let port = tcp.port.unwrap();
        let family = &tcp.family;

        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            let socket_addr = SocketAddr::new(ip_addr, port);
            match Stream::tcp_connect(&socket_addr, family).await {
                Ok(tcp_stream) => Ok(Stream::Tcp(tcp_stream)),
                Err(e) => {
                    error!("Could not connec to {}: {}", socket_addr, e);
                    Err(ConnectError::TcpResolveIpAddress)
                }
            }
        } else {
            let host_port = format!("{}:{}", host, port);
            for socket_addr in lookup_host(host_port).await? {
                match Stream::tcp_connect(&socket_addr, family).await {
                    Ok(tcp_stream) => return Ok(Stream::Tcp(tcp_stream)),
                    Err(e) => error!("Could not connect to {}: {}", socket_addr, e),
                }
            }

            Err(ConnectError::TcpResolveIpAddress)
        }
    }

    async fn connect(address: &Address) -> Result<Stream, ConnectError> {
        if !address.is_connectable() {
            return Err(ConnectError::TcpAddressNotConnectable);
        }

        match address {
            Address::Unix(unix) => Stream::unix(unix).await,
            Address::Tcp(tcp) => Stream::tcp(tcp).await,
            Address::NonceTcp(_) => Err(ConnectError::NonceTcpNotSupported),
            x => panic!("This should not happen: {}", x),
        }
    }

    /// Get the Unix Domain Stream socket by connection to the socket defined in the
    /// `DBUS_SESSION_BUS_ADDRESS` environment variable.
    pub async fn new(addressses: &str) -> Result<(Address, Stream), StreamError> {
        let addressses = Address::decode(addressses)?;
        for address in addressses.iter() {
            match Stream::connect(address).await {
                Ok(connect) => return Ok((address.clone(), connect)),
                Err(e) => {
                    error!("Could not connect to {}: {}", address, e);
                }
            }
        }
        // It could not connect to any socket
        Err(StreamError::ConnectError(
            ConnectError::CouldNotConnectToAnyAddress,
        ))
    }
}
