use crate::command::Command;
use crate::connection::Connection;
use crate::error::DBusResult;
use crate::helper::{get_unix_socket, split};
use crate::introspect::add_introspect;
use crate::message::{message_sink, message_stream};
use crate::DBusNameFlag;
use dbus_message_parser::{Message, MessageType, Value};
use futures::channel::mpsc::{unbounded, Sender as MpscSender, UnboundedSender};
use futures::channel::oneshot::channel;
use std::collections::HashSet;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

/// This struct represents an object to communicate with the DBus daemon.
#[derive(Debug, Clone)]
pub struct DBus {
    command_sender: UnboundedSender<Command>,
    socket_path: Arc<String>,
}

impl DBus {
    /// Connect to the session DBus.
    pub async fn session(introspectable: bool) -> IoResult<(DBus, JoinHandle<()>)> {
        if let Some(path) = option_env!("DBUS_SESSION_BUS_ADDRESS") {
            DBus::new(path, introspectable).await
        } else {
            // It could not connect to any socket
            Err(IoError::new(
                IoErrorKind::Other,
                "DBUS_SESSION_BUS_ADDRESS environment variable is not defined",
            ))
        }
    }

    /// Connect to the system DBus.
    pub async fn system(introspectable: bool) -> IoResult<(DBus, JoinHandle<()>)> {
        let path = if let Some(path) = option_env!("DBUS_SYSTEM_BUS_ADDRESS") {
            path
        } else {
            "unix:path=/var/run/dbus/system_bus_socket"
        };
        DBus::new(path, introspectable).await
    }

    /// Create a DBus object. You can choose in the second argument that the Peer is introspectable.
    pub async fn new(path: &str, introspectable: bool) -> IoResult<(DBus, JoinHandle<()>)> {
        // Create all necessary channels.
        let (command_sender, command_receiver) = unbounded::<Command>();
        let (message_sender, message_receiver) = unbounded::<Message>();

        let socket = get_unix_socket(path).await?;
        let (sink, stream) = split(socket)?;

        // Spawn the sink task.
        spawn(message_sink(message_receiver, sink));
        // Spawn the stream task.
        spawn(message_stream(stream, command_sender.clone()));
        // Spawn the connection task.
        let connection = Connection::from(command_receiver, message_sender);
        let connection_handle = spawn(connection.run());

        let socket_path = Arc::new(path.to_string());
        let dbus = DBus {
            command_sender,
            socket_path,
        };
        if introspectable {
            add_introspect(dbus.clone())?;
        }

        // Send the Hello message.
        let msg = dbus.call_hello().await?;
        if let MessageType::Error = msg.get_type() {
            let error = if let Some(error) = msg.get_error_name() {
                error.as_ref()
            } else {
                "no error name"
            };
            let error = format!("call hello: {}", error);
            Err(IoError::new(IoErrorKind::Other, error))
        } else {
            Ok((dbus, connection_handle))
        }
    }

    /// Send a `Message` without waiting for a response.
    pub fn send(&self, msg: Message) -> DBusResult<()> {
        // Try to send the message.
        let command = Command::SendMessage(msg, None);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Send a `Message` with waiting for a response.
    pub async fn call(&self, msg: Message) -> DBusResult<Message> {
        // Create a oneshot channel for the response
        let (msg_sender, msg_receiver) = channel::<Message>();
        // Try to send the message.
        let command = Command::SendMessage(msg, Some(msg_sender));
        self.command_sender.unbounded_send(command)?;
        let msg = msg_receiver.await?;
        Ok(msg)
    }

    /// Send the Hello `Message` and wait for the response.
    async fn call_hello(&self) -> DBusResult<Message> {
        let msg = Message::method_call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
        );
        self.call(msg).await
    }

    /// Register a name for the peer.
    /// This calls the `RequestName` method from the DBus daemon.
    pub async fn register_name(&self, name: String, flags: &DBusNameFlag) -> DBusResult<Message> {
        let mut msg = Message::method_call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "RequestName",
        );
        msg.add_value(Value::String(name));
        msg.add_value(Value::Uint32(flags.bits));
        self.call(msg).await
    }

    /// Add a `Handler` to a specific path.
    pub fn add_object_path(
        &self,
        object_path: String,
        sender: MpscSender<Message>,
    ) -> DBusResult<()> {
        let command = Command::AddPath(object_path, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete a object by path.
    pub fn delete_object_path(&self, object_path: String) -> DBusResult<()> {
        let command = Command::DeletePath(object_path);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete a object by sender.
    pub fn delete_sender(&self, sender: MpscSender<Message>) -> DBusResult<()> {
        let command = Command::DeleteSender(sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Add an interface `Handler`.
    pub fn add_interface(&self, interface: String, sender: MpscSender<Message>) -> DBusResult<()> {
        let command = Command::AddInterface(interface, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Add a signal handler.
    pub fn add_signal_handler(&self, path: String, sender: MpscSender<Message>) -> DBusResult<()> {
        let command = Command::AddSignalHandler(path, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// List all objects under a specific path.
    pub async fn list_path(&self, path: &str) -> DBusResult<HashSet<String>> {
        let (sender, receiver) = channel();
        let command = Command::ListPath(path.to_string(), sender);
        self.command_sender.unbounded_send(command)?;
        let list = receiver.await?;
        Ok(list)
    }

    /// Close the DBus object.
    pub fn close(&self) -> DBusResult<()> {
        self.command_sender.unbounded_send(Command::Close)?;
        Ok(())
    }

    /// The current path of the DBus socket.
    pub fn get_socket_path(&self) -> &str {
        self.socket_path.as_ref()
    }
}
