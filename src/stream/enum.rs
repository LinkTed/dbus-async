use super::{
    handshake::HandshakeError,
    message::{message_sink, message_stream},
};
use dbus_message_parser::message::Message;
use dbus_server_address_parser::DecodeError;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use std::{io::Error as IoError, str::Utf8Error};
use thiserror::Error;
use tokio::{
    net::{TcpStream, UnixStream},
    spawn,
};

#[derive(Debug)]
pub enum Stream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Could not parse address: {0}")]
    DecodeError(#[from] DecodeError),
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
    #[error("Autolaunch is currently not supported")]
    AutolaunchNotSupported,
    #[error("Launchd is currently not supported")]
    LaunchdNotSupported,
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
    #[error("Handshake Error: {0}")]
    HandshakeError(#[from] HandshakeError),
    #[error("Printed path is not UTF-8")]
    UnixexecStdout(Utf8Error),
}

impl Stream {
    pub fn start(self) -> (UnboundedSender<Message>, UnboundedReceiver<Message>) {
        // Create all necessary channels.
        let (message_sink_sender, message_sink_receiver) = unbounded::<Message>();
        let (message_stream_sender, message_stream_receiver) = unbounded::<Message>();

        match self {
            Stream::Unix(unix_stream) => {
                let (stream, sink) = unix_stream.into_split();
                // Spawn the sink task.
                spawn(message_stream(stream, message_stream_sender));
                // Spawn the stream task.
                spawn(message_sink(message_sink_receiver, sink));
            }
            Stream::Tcp(tcp_stream) => {
                let (stream, sink) = tcp_stream.into_split();
                // Spawn the sink task.
                spawn(message_stream(stream, message_stream_sender));
                // Spawn the stream task.
                spawn(message_sink(message_sink_receiver, sink));
            }
        }

        (message_sink_sender, message_stream_receiver)
    }
}
