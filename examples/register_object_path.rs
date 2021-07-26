use dbus_async::DBus;
use dbus_message_parser::message::Message;
use futures::{channel::mpsc::channel, stream::StreamExt};
use std::convert::TryInto;

// This is a low level example, where the user create the channel to receive the message.

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true, true)
        .await
        .expect("failed to get the DBus object");

    // Initialize the object path.
    let object_path = "/object/path/test".try_into().unwrap();

    // Create a FIFO with a size of 1024
    let (sender, mut receiver) = channel::<Message>(1024);

    // Register the object path
    if let Err(e) = dbus.add_method_call(object_path, sender) {
        panic!("Cannot add path: {:?}", e);
    }

    // Get the next message for the object path "/object/path/test"
    while let Some(msg) = receiver.next().await {
        println!("{:?}", msg);
    }
}
