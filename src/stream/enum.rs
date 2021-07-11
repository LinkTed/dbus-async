use super::{
    connect::ConnectError,
    message::{message_sink, message_stream},
};
use dbus_message_parser::message::Message;
use dbus_server_address_parser::DecodeError;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use std::io::Error as IoError;
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
    #[error("IO error: {0}")]
    IoError(#[from] IoError),
    #[error("Got the following response from daemon: {0}")]
    HandshakeOk(String),
    #[error("Got the following response from daemon: {0}")]
    HandshakeUnixFD(String),
    #[error("Could not connect: {0}")]
    ConnectError(#[from] ConnectError),
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
