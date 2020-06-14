use dbus_async::DBus;
use dbus_message_parser::Message;

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall
    let msg = Message::method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.Peer",
        "Ping",
    );

    // Send the message and get the return message
    let return_msg = dbus.call(msg).await;

    // Print the return message
    println!("{:?}", return_msg);
}
