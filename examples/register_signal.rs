use dbus_async::DBus;
use dbus_message_parser::message::Message;
use dbus_message_parser::value::Value;
use futures::channel::mpsc::channel;
use futures::stream::StreamExt;
use std::convert::TryInto;

// This is a low level example, where the user create the channel to receive signals from specific
// peer.
#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true, true)
        .await
        .expect("failed to get the DBus object");

    // Add to match rule to get all signals from "org.freedesktop.DBus" sender and with a object
    // path of "/org/freedesktop/DBus"
    let mut msg_add_match = Message::method_call(
        "org.freedesktop.DBus".try_into().unwrap(),
        "/org/freedesktop/DBus".try_into().unwrap(),
        "org.freedesktop.DBus".try_into().unwrap(),
        "AddMatch".try_into().unwrap(),
    );
    msg_add_match.add_value(Value::String(
        "type='signal',sender='org.freedesktop.DBus',\
            path='/org/freedesktop/DBus',interface='org.freedesktop.DBus'"
            .to_string(),
    ));
    dbus.call(msg_add_match)
        .await
        .expect("Could not add match rule");

    // Initialize the object path
    let object_path = "org/freedesktop/DBus".try_into().unwrap();

    // Create a FIFO with a size of 1024
    let (sender, mut receiver) = channel::<Message>(1024);

    // Register the object path
    if let Err(e) = dbus.add_signal(object_path, None, sender) {
        panic!("Cannot add path: {:?}", e);
    }

    // Get the next signal from the object path "org.freedesktop.DBus"
    while let Some(msg) = receiver.next().await {
        println!("{:?}", msg);
    }
}
