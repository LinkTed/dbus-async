use super::{
    handshake::{Handshake, HandshakeError, Nonce},
    Stream, StreamError,
};
use dbus_server_address_parser::{Address, Family, NonceTcp, Tcp, Unix, UnixType};
use std::{
    io::Error as IoError,
    net::{IpAddr, SocketAddr},
};
use thiserror::Error;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    net::{lookup_host, TcpStream, UnixStream},
};

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Unix abstract is not yet supported")]
    UnixAbstractNotSupported,
    #[error("Could not connect to any address")]
    CouldNotConnectToAnyAddress,
    #[error("Address is not connectable")]
    AddressNotConnectable,
    #[error("Could not resolve IP addresses, which match the given IP family")]
    TcpResolveIpAddress,
    #[error("Noncefile is too large")]
    NonceTcpFileTooLarge,
    #[error("Noncefile is too small")]
    NonceTcpFileTooSmall,
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
                Handshake::handshake(&mut connection, true, &None).await?;
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

    async fn tcp_connect_address(
        socket_addr: &SocketAddr,
        family: &Option<Family>,
        nonce: &Option<Nonce>,
    ) -> Result<TcpStream, ConnectError> {
        if !Stream::tcp_family_match(socket_addr, family) {
            return Err(ConnectError::TcpResolveIpAddress);
        }

        debug!("Connect to {}", socket_addr);
        let mut tcp_stream = TcpStream::connect(socket_addr).await?;
        Handshake::handshake(&mut tcp_stream, false, nonce).await?;
        Ok(tcp_stream)
    }

    async fn tcp_connect(
        host: &str,
        port: u16,
        family: &Option<Family>,
        nonce: &Option<Nonce>,
    ) -> Result<Stream, ConnectError> {
        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            let socket_addr = SocketAddr::new(ip_addr, port);
            match Stream::tcp_connect_address(&socket_addr, family, nonce).await {
                Ok(tcp_stream) => Ok(Stream::Tcp(tcp_stream)),
                Err(e) => {
                    error!("Could not connect to {}: {}", socket_addr, e);
                    Err(ConnectError::TcpResolveIpAddress)
                }
            }
        } else {
            let host_port = format!("{}:{}", host, port);
            for socket_addr in lookup_host(host_port).await? {
                match Stream::tcp_connect_address(&socket_addr, family, nonce).await {
                    Ok(tcp_stream) => return Ok(Stream::Tcp(tcp_stream)),
                    Err(e) => error!("Could not connect to {}: {}", socket_addr, e),
                }
            }

            Err(ConnectError::TcpResolveIpAddress)
        }
    }

    async fn tcp(tcp: &Tcp) -> Result<Stream, ConnectError> {
        let host = tcp.host.as_ref().unwrap();
        let port = tcp.port.unwrap();
        let family = &tcp.family;

        Stream::tcp_connect(host, port, family, &None).await
    }

    async fn nonce_tcp_read_nonce(nonce_tcp: &NonceTcp) -> Result<Nonce, ConnectError> {
        let mut nonce: Nonce = [0; 16];

        let noncefile = nonce_tcp.noncefile.as_ref().unwrap();
        let mut noncefile = File::open(noncefile).await?;

        let read = noncefile.read_exact(&mut nonce).await?;
        if read == nonce.len() {
            // Check if there is more bytes
            let read = noncefile.read(&mut nonce).await?;
            if read == 0 {
                Ok(nonce)
            } else {
                Err(ConnectError::NonceTcpFileTooLarge)
            }
        } else {
            Err(ConnectError::NonceTcpFileTooSmall)
        }
    }

    async fn nonce_tcp(nonce_tcp: &NonceTcp) -> Result<Stream, ConnectError> {
        let host = nonce_tcp.host.as_ref().unwrap();
        let port = nonce_tcp.port.unwrap();
        let family = &nonce_tcp.family;

        let nonce = Stream::nonce_tcp_read_nonce(nonce_tcp).await?;
        let nonce = Some(nonce);

        Stream::tcp_connect(host, port, family, &nonce).await
    }

    async fn connect(address: &Address) -> Result<Stream, ConnectError> {
        if !address.is_connectable() {
            return Err(ConnectError::AddressNotConnectable);
        }

        match address {
            Address::Unix(unix) => Stream::unix(unix).await,
            Address::Tcp(tcp) => Stream::tcp(tcp).await,
            Address::NonceTcp(nonce_tcp) => Stream::nonce_tcp(nonce_tcp).await,
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
        // It could not connect to any address
        Err(StreamError::ConnectError(
            ConnectError::CouldNotConnectToAnyAddress,
        ))
    }
}
