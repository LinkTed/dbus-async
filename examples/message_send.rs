use dbus_async::DBus;
use dbus_message_parser::message::Message;
use std::convert::TryInto;

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Now we have a DBus object, so create a message
    let msg = Message::method_call(
        "org.freedesktop.DBus".try_into().unwrap(),
        "/org/freedesktop/DBus".try_into().unwrap(),
        "org.freedesktop.DBus.Peer".try_into().unwrap(),
        "Ping".try_into().unwrap(),
    );

    // Send the message
    let result = dbus.send(msg);
    println!("{}", result.is_ok());
}
