use dbus_async::DBus;
use dbus_message_parser::message::Message;
use std::convert::TryInto;

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true, true)
        .await
        .expect("failed to get the DBus object");

    // Now we have a DBus object, so create a message
    let msg = Message::signal(
        "/org/example/DBus".try_into().unwrap(),
        "org.example.DBus.Peer".try_into().unwrap(),
        "ExampleSignal".try_into().unwrap(),
    );

    // Send the message
    let result = dbus.send(msg);
    println!("{}", result.is_ok());
}
