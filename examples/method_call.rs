use dbus_async::DBus;
use dbus_message_parser::message::Message;
use std::convert::TryInto;

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true, true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall
    let msg = Message::method_call(
        "org.freedesktop.DBus".try_into().unwrap(),
        "/org/freedesktop/DBus".try_into().unwrap(),
        "org.freedesktop.DBus.Peer".try_into().unwrap(),
        "Ping".try_into().unwrap(),
    );

    // Send the message and get the return message
    let return_msg = dbus.call(msg).await;

    // Print the return message
    println!("{:?}", return_msg);
}
