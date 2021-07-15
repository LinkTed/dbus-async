use super::{
    handshake::{Handshake, Nonce},
    Stream, StreamError,
};
use async_recursion::async_recursion;
use dbus_server_address_parser::{Address, Family, NonceTcp, Tcp, Unix, UnixType, Unixexec};
use std::{
    net::{IpAddr, SocketAddr},
    str::from_utf8,
};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    net::{lookup_host, TcpStream, UnixStream},
    process::Command,
};

impl Stream {
    async fn unix(unix: &Unix) -> Result<Stream, StreamError> {
        match &unix.r#type {
            UnixType::Path(path) => {
                debug!("Connect to {}", path);
                let mut connection = UnixStream::connect(path).await?;
                Handshake::handshake(&mut connection, true, &None).await?;
                Ok(Stream::Unix(connection))
            }
            UnixType::Abstract(_) => Err(StreamError::UnixAbstractNotSupported),
            x => panic!("This should not happen: {}", x),
        }
    }

    #[async_recursion]
    async fn unixexec(unixexec: &Unixexec) -> Result<Stream, StreamError> {
        // TODO: missing argv0 support by the Tokio API
        let output = Command::new(&unixexec.path)
            .args(&unixexec.argv)
            .output()
            .await?;
        match from_utf8(&output.stdout) {
            Ok(addressses) => {
                let (_, stream) = Stream::new(addressses).await?;
                Ok(stream)
            }
            Err(e) => Err(StreamError::UnixexecStdout(e)),
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
    ) -> Result<TcpStream, StreamError> {
        if !Stream::tcp_family_match(socket_addr, family) {
            return Err(StreamError::TcpResolveIpAddress);
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
    ) -> Result<Stream, StreamError> {
        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            let socket_addr = SocketAddr::new(ip_addr, port);
            match Stream::tcp_connect_address(&socket_addr, family, nonce).await {
                Ok(tcp_stream) => Ok(Stream::Tcp(tcp_stream)),
                Err(e) => {
                    error!("Could not connect to {}: {}", socket_addr, e);
                    Err(StreamError::TcpResolveIpAddress)
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

            Err(StreamError::TcpResolveIpAddress)
        }
    }

    async fn tcp(tcp: &Tcp) -> Result<Stream, StreamError> {
        let host = tcp.host.as_ref().unwrap();
        let port = tcp.port.unwrap();
        let family = &tcp.family;

        Stream::tcp_connect(host, port, family, &None).await
    }

    async fn nonce_tcp_read_nonce(nonce_tcp: &NonceTcp) -> Result<Nonce, StreamError> {
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
                Err(StreamError::NonceTcpFileTooLarge)
            }
        } else {
            Err(StreamError::NonceTcpFileTooSmall)
        }
    }

    async fn nonce_tcp(nonce_tcp: &NonceTcp) -> Result<Stream, StreamError> {
        let host = nonce_tcp.host.as_ref().unwrap();
        let port = nonce_tcp.port.unwrap();
        let family = &nonce_tcp.family;

        let nonce = Stream::nonce_tcp_read_nonce(nonce_tcp).await?;
        let nonce = Some(nonce);

        Stream::tcp_connect(host, port, family, &nonce).await
    }

    async fn connect(address: &Address) -> Result<Stream, StreamError> {
        if !address.is_connectable() {
            return Err(StreamError::AddressNotConnectable);
        }

        match address {
            Address::Unix(unix) => Stream::unix(unix).await,
            Address::Unixexec(unixexec) => Stream::unixexec(unixexec).await,
            Address::Tcp(tcp) => Stream::tcp(tcp).await,
            Address::NonceTcp(nonce_tcp) => Stream::nonce_tcp(nonce_tcp).await,
            Address::Autolaunch(_) => Err(StreamError::AutolaunchNotSupported),
            Address::Launchd(_) => Err(StreamError::LaunchdNotSupported),
            x => panic!("This should not happen: {}", x),
        }
    }

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
        Err(StreamError::CouldNotConnectToAnyAddress)
    }
}
