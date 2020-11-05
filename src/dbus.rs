use crate::command::Command;
use crate::connection::Connection;
use crate::error::DBusResult;
use crate::helper::{get_unix_socket, split};
use crate::introspect::add_introspect;
use crate::message::{message_sink, message_stream};
use crate::DBusNameFlag;
use dbus_message_parser::{Interface, Message, MessageType, ObjectPath, Value};
use futures::channel::mpsc::{
    unbounded, Receiver as MpscReceiver, Sender as MpscSender, UnboundedSender,
};
use futures::channel::oneshot::channel;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

/// This struct represents an object to communicate with the DBus daemon.
#[derive(Clone)]
pub struct DBus {
    command_sender: UnboundedSender<Command>,
    socket_path: Arc<String>,
}

impl DBus {
    /// Connect to the session DBus.
    ///
    /// If the first argument (`introspectable`) is `true` then the Peer is introspectable.
    ///
    /// The `DBUS_SESSION_BUS_ADDRESS` environment variable **have to** be defined.
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
    ///
    /// If the first argument (`introspectable`) is `true` then the Peer is introspectable.
    ///
    /// If there `DBUS_SYSTEM_BUS_ADDRESS` environment variable is defined then this path will be
    /// used, else `unix:path=/var/run/dbus/system_bus_socket`.
    pub async fn system(introspectable: bool) -> IoResult<(DBus, JoinHandle<()>)> {
        let path = if let Some(path) = option_env!("DBUS_SYSTEM_BUS_ADDRESS") {
            path
        } else {
            "unix:path=/var/run/dbus/system_bus_socket"
        };
        DBus::new(path, introspectable).await
    }

    /// Connect to the specific (`path`) DBus daemon.
    ///
    /// If the second argument (`introspectable`) is `true` then the Peer is introspectable.
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
                error
            } else {
                "no error name"
            };
            let error = format!("call hello: {}", error);
            Err(IoError::new(IoErrorKind::Other, error))
        } else {
            Ok((dbus, connection_handle))
        }
    }

    /// Send a [`Message`](dbus_message_parser::Message).
    pub fn send(&self, msg: Message) -> DBusResult<()> {
        // Try to send the message.
        let command = Command::SendMessage(msg);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Send a [`Message`] and wait for a response.
    ///
    /// The [`Message`] have to be a `MessageCall`.
    ///
    /// [`Message`]: dbus_message_parser::Message
    pub async fn call(&self, msg: Message) -> DBusResult<Message> {
        // Create a oneshot channel for the response
        let (msg_sender, msg_receiver) = channel::<Message>();
        // Try to send the message.
        let command = Command::SendMessageOneshot(msg, msg_sender);
        self.command_sender.unbounded_send(command)?;
        let msg = msg_receiver.await?;
        Ok(msg)
    }

    /// Send a [`Message`] and specify a channel, where the response should be send.
    ///
    /// This function returns the serial number of the [`Message`]. This is useful, where the the
    /// response and signals have to be processed in order.
    ///
    /// [`Message`]: dbus_message_parser::Message
    pub async fn call_reply_serial(
        &self,
        msg: Message,
        msg_sender: MpscSender<Message>,
    ) -> DBusResult<u32> {
        let (reply_serial_sender, reply_serial_receiver) = channel::<u32>();
        // Try to send the message.
        let command = Command::SendMessageMpcs(msg, reply_serial_sender, msg_sender);
        self.command_sender.unbounded_send(command)?;
        let reply_serial = reply_serial_receiver.await?;
        Ok(reply_serial)
    }

    /// Call the `Hello()` method of the DBus daemon.
    async fn call_hello(&self) -> DBusResult<Message> {
        let msg = Message::method_call(
            "org.freedesktop.DBus".try_into().unwrap(),
            "/org/freedesktop/DBus".try_into().unwrap(),
            "org.freedesktop.DBus".try_into().unwrap(),
            "Hello".try_into().unwrap(),
        );
        self.call(msg).await
    }

    /// Register a name for the peer. This calls the `RequestName(String, UInt32)` method of the
    /// DBus daemon.
    pub async fn register_name(&self, name: String, flags: &DBusNameFlag) -> DBusResult<Message> {
        let mut msg = Message::method_call(
            "org.freedesktop.DBus".try_into().unwrap(),
            "/org/freedesktop/DBus".try_into().unwrap(),
            "org.freedesktop.DBus".try_into().unwrap(),
            "RequestName".try_into().unwrap(),
        );
        msg.add_value(Value::String(name));
        msg.add_value(Value::Uint32(flags.bits));
        self.call(msg).await
    }

    /// Add a channel to a specific [`ObjectPath`].
    ///
    /// The channel will receive all `MethodCall` messages for the specified [`ObjectPath`].
    ///
    /// If there is already channel added for this [`ObjectPath`] then it will be replace. So the
    /// old channel will not receive any `MethodCall` messages for the [`ObjectPath`] anymore.
    ///
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn add_method_call(
        &self,
        object_path: ObjectPath,
        sender: MpscSender<Message>,
    ) -> DBusResult<()> {
        let command = Command::AddMethodCall(object_path, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for a specific [`ObjectPath`] (see [`add_method_call`]).
    ///
    /// Even if there is no channel for this [`ObjectPath`] the function will return `Ok()`.
    ///
    /// [`add_method_call`]: #method.add_method_call
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn delete_object_path(&self, object_path: ObjectPath) -> DBusResult<()> {
        let command = Command::DeleteMethodCall(object_path);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`ObjectPath`], which the given sender is connected to
    /// (see [`add_method_call`]).
    ///
    /// [`add_method_call`]: #method.add_method_call
    pub fn delete_method_call_sender(&self, sender: MpscSender<Message>) -> DBusResult<()> {
        let command = Command::DeleteMethodCallSender(sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`ObjectPath`], which the given sender is connected to
    /// (see [`add_method_call`]).
    ///
    /// [`add_method_call`]: #method.add_method_call
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn delete_method_call_receiver(&self, receiver: MpscReceiver<Message>) -> DBusResult<()> {
        let command = Command::DeleteMethodCallReceiver(receiver);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Add a channel to a specific [`Interface`].
    ///
    /// The channel will **only** receive all `MethodCall` messages for the specified [`Interface`],
    /// if there is no channel by the [`ObjectPath`].
    ///
    /// If there is already channel added for this [`Interface`] then it will be replace. So the old
    /// channel will not receive any `MethodCall` messages for the [`Interface`] anymore.
    ///
    /// [`Interface`]: dbus_message_parser::Interface
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn add_method_call_interface(
        &self,
        interface: Interface,
        sender: MpscSender<Message>,
    ) -> DBusResult<()> {
        let command = Command::AddMethodCallInterface(interface, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`Interface`], which the given sender is connected to
    /// (see [`add_method_call_interface`]).
    ///
    /// [`add_method_call_interface`]: #method.add_method_call_interface
    /// [`Interface`]: dbus_message_parser::Interface
    pub fn delete_method_call_interface_sender(
        &self,
        sender: MpscSender<Message>,
    ) -> DBusResult<()> {
        let command = Command::DeleteMethodCallInterfaceSender(sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`Interface`], which the given sender is connected to
    /// (see [`add_method_call_interface`]).
    ///
    /// [`add_method_call_interface`]: #method.add_method_call_interface
    /// [`Interface`]: dbus_message_parser::Interface
    pub fn delete_method_call_interface_receiver(
        &self,
        receiver: MpscReceiver<Message>,
    ) -> DBusResult<()> {
        let command = Command::DeleteMethodCallInterfaceReceiver(receiver);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Add a channel to a specific [`ObjectPath`].
    ///
    /// The channel will receive all `Signal` messages for the specified [`ObjectPath`].
    ///
    /// The second argument specify a closure to filter the [`Message`]. If the closure returns true
    /// then the [`Message`] will not be send to the channel.
    ///
    /// There can be multiple channels, which will receive message of the specific [`ObjectPath`].
    ///
    /// [`Message`]: dbus_message_parser::Message
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn add_signal(
        &self,
        object_path: ObjectPath,
        filter: Option<fn(&Message) -> bool>,
        sender: MpscSender<Message>,
    ) -> DBusResult<()> {
        let command = Command::AddSignal(object_path, filter, sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`ObjectPath`], which the given sender is connected to
    /// (see [`add_signal`]).
    ///
    /// [`add_signal`]: #method.add_signal
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn delete_signal_sender(&self, sender: MpscSender<Message>) -> DBusResult<()> {
        let command = Command::DeleteSignalSender(sender);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// Delete the channel for every [`ObjectPath`], which the given sender is connected to
    /// (see [`add_signal`]).
    ///
    /// [`add_signal`]: #method.add_signal
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub fn delete_signal_receiver(&self, receiver: MpscReceiver<Message>) -> DBusResult<()> {
        let command = Command::DeleteSignalReceiver(receiver);
        self.command_sender.unbounded_send(command)?;
        Ok(())
    }

    /// List all [`ObjectPath`]s under the given [`ObjectPath`].
    ///
    /// This will only list the [`ObjectPath`] for the `MethodCall` messages
    /// (see [`add_method_call`]).
    ///
    /// [`add_method_call`]: #method.add_method_call
    /// [`ObjectPath`]: dbus_message_parser::ObjectPath
    pub async fn list_method_call(&self, object_path: ObjectPath) -> DBusResult<HashSet<String>> {
        let (sender, receiver) = channel();
        let command = Command::ListMethodCall(object_path, sender);
        self.command_sender.unbounded_send(command)?;
        let list = receiver.await?;
        Ok(list)
    }

    /// Close the DBus connection.
    pub fn close(&self) -> DBusResult<()> {
        self.command_sender.unbounded_send(Command::Close)?;
        Ok(())
    }

    /// Get the current path of the DBus daemon.
    pub fn get_socket_path(&self) -> &str {
        self.socket_path.as_ref()
    }
}
