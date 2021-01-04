use crate::command::Command;
use bytes::{Buf, BytesMut};
use dbus_message_parser::decode::DecodeError;
use dbus_message_parser::message::Message;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The message sink task. This task takes messages from the channel and send it through the DBus
/// socket.
pub async fn message_sink<T>(mut message_receiver: UnboundedReceiver<Message>, mut sink: T)
where
    T: AsyncWriteExt + Unpin,
{
    // Get the next Message to send to the DBus socket
    let mut buffer = BytesMut::new();
    while let Some(msg) = message_receiver.next().await {
        // Try to encode
        if let Err(e) = msg.encode() {
            error!("message_sink: {:?}", e);
            return;
        }

        match sink.write(buffer.as_mut()).await {
            Ok(size) => {
                buffer.advance(size);
            }
            Err(e) => {
                error!("message_sink: {:?}", e);
                return;
            }
        }

        while !buffer.is_empty() {
            match sink.write(buffer.as_mut()).await {
                Ok(size) => {
                    buffer.advance(size);
                }
                Err(e) => {
                    error!("message_sink: {:?}", e);
                    return;
                }
            }
        }
    }
}

/// The message stream task. This task takes messages, which were received from the DBus socket.
pub async fn message_stream<T>(mut stream: T, command_sender: UnboundedSender<Command>)
where
    T: AsyncReadExt + Unpin,
{
    let mut buffer_msg = BytesMut::new();
    // Get the next Message received from the DBus socket
    let mut buffer: [u8; 128] = [0; 128];
    loop {
        match stream.read(&mut buffer[..]).await {
            Ok(size) => {
                buffer_msg.extend_from_slice(&buffer[..size]);
            }
            Err(e) => {
                error!("message_stream: {:?}", e);
                return;
            }
        }

        while !buffer_msg.is_empty() {
            let bytes = buffer_msg.clone().freeze();
            let result = Message::decode(bytes);
            match result {
                Ok((msg, offset)) => {
                    buffer_msg.advance(offset);
                    // Try to send the message to the server
                    if let Err(e) = command_sender.unbounded_send(Command::ReceiveMessage(msg)) {
                        error!("message_stream: {}", e);
                        return;
                    }
                }
                Err(DecodeError::NotEnoughBytes(u1, u2)) => {
                    debug!(
                        "message_stream: DecodeError::NotEnoughBytes({}, {})",
                        u1, u2
                    );
                    break;
                }
                Err(e) => {
                    error!("message_stream: {:?}", e);
                    return;
                }
            }
        }
    }
}
