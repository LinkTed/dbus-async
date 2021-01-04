use crate::command::Command;
use crate::message::{message_sink, message_stream};
use crate::{ServerAddress, ServerAddressParseError};
use dbus_message_parser::message::Message;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use hex::encode;
use std::io::Error as IoError;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufStream};
use tokio::net::{TcpStream, UnixStream};
use tokio::spawn;

/// Connect to the Unix Domain Stream socket.
async fn handshake<T>(stream: &mut T) -> Result<(), StreamError>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    let mut stream = BufStream::new(stream);
    // Connect to the Unix Domain Stream.
    // let mut stream = TokioUnixStream::connect(path).await?;
    // Write a zero to the socket.
    let zero: [u8; 1] = [0; 1];
    stream.write_all(&zero[..]).await?;
    // Get the UID of the process
    let uid = unsafe { libc::getuid() };
    // Encode the UID in a hex string.
    let hex = encode(uid.to_string());
    // Authenticate to the DBus daemon.
    let cmd = "AUTH EXTERNAL ".to_owned() + &hex + "\r\n";
    stream.write_all(&cmd.into_bytes()).await?;
    stream.flush().await?;

    // Read the response of the socket.
    let mut resp = String::new();
    stream.read_line(&mut resp).await?;
    // Check if the authentication is successful.
    if !resp.starts_with("OK ") {
        return Err(StreamError::HandshakeOk(resp));
    }

    let cmd = "NEGOTIATE_UNIX_FD\r\n";
    stream.write_all(cmd.as_bytes()).await?;
    stream.flush().await?;

    resp.clear();
    stream.read_line(&mut resp).await?;
    // Check if the authentication is successful.
    if resp != "AGREE_UNIX_FD\r\n" {
        return Err(StreamError::HandshakeUnixFD(resp));
    }

    // Authentication was successful.
    stream.write_all(b"BEGIN\r\n").await?;
    stream.flush().await?;
    Ok(())
}

#[derive(Debug)]
pub enum Stream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Could not parse address: {0}")]
    ServerAddressParseError(#[from] ServerAddressParseError),
    #[error("IO error: {0}")]
    IoError(#[from] IoError),
    #[error("Unix abstract is not yet supported")]
    UnixAbstractIsNotSupported,
    #[error("Unix runtime is not yes supported")]
    UnixRuntimeIsNotSupported,
    #[error("Could not connect to any address")]
    CouldNotConnectToAnyAddress,
    #[error("Got the following response from daemon: {0}")]
    HandshakeOk(String),
    #[error("Got the following response from daemon: {0}")]
    HandshakeUnixFD(String),
}

impl Stream {
    async fn connect(address: &ServerAddress) -> Result<Stream, StreamError> {
        match address {
            ServerAddress::UnixPath(path) => {
                let mut connection = UnixStream::connect(path).await?;
                handshake(&mut connection).await?;
                Ok(Stream::Unix(connection))
            }
            ServerAddress::UnixAbstract(_) => Err(StreamError::UnixAbstractIsNotSupported),
            ServerAddress::UnixRuntime => Err(StreamError::UnixRuntimeIsNotSupported),
            ServerAddress::Tcp(socket_address) => {
                let mut connection = TcpStream::connect(socket_address).await?;
                handshake(&mut connection).await?;
                Ok(Stream::Tcp(connection))
            }
        }
    }

    /// Get the Unix Domain Stream socket by connection to the socket defined in the
    /// `DBUS_SESSION_BUS_ADDRESS` environment variable.
    pub async fn new(addressses: &str) -> Result<(ServerAddress, Stream), StreamError> {
        let addressses = ServerAddress::parse(addressses)?;
        for address in addressses.iter() {
            match Stream::connect(address).await {
                Ok(connect) => return Ok((address.clone(), connect)),
                Err(e) => {
                    error!("Could not connect to {}: {}", address, e);
                }
            }
        }
        // It could not connect to any socket
        Err(StreamError::CouldNotConnectToAnyAddress)
    }

    pub fn start(self, command_sender: UnboundedSender<Command>) -> UnboundedSender<Message> {
        // Create all necessary channels.
        let (message_sender, message_receiver) = unbounded::<Message>();

        match self {
            Stream::Unix(unix_stream) => {
                let (stream, sink) = unix_stream.into_split();
                // Spawn the sink task.
                spawn(message_stream(stream, command_sender));
                // Spawn the stream task.
                spawn(message_sink(message_receiver, sink));
            }
            Stream::Tcp(tcp_stream) => {
                let (stream, sink) = tcp_stream.into_split();
                // Spawn the sink task.
                spawn(message_stream(stream, command_sender));
                // Spawn the stream task.
                spawn(message_sink(message_receiver, sink));
            }
        }

        message_sender
    }
}
