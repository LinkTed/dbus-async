use dbus_async::DBus;
use dbus_message_parser::{Message, MessageType, Value};
use futures::channel::mpsc::{channel, Sender};
use futures::stream::StreamExt;

async fn register_signal(dbus: &DBus, sender: Sender<Message>) {
    let mut msg_add_match = Message::method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "AddMatch",
    );
    msg_add_match.add_value(Value::String(
        "type='signal',sender='org.example.sender',\
            path='/',interface='org.freedesktop.DBus.ObjectManager.InterfacesAdded'"
            .to_string(),
    ));
    dbus.call(msg_add_match)
        .await
        .expect("Could not add match rule");

    let mut msg_add_match = Message::method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "AddMatch",
    );
    msg_add_match.add_value(Value::String(
        "type='signal',sender='org.example.sender',\
            path='/',interface='org.freedesktop.DBus.ObjectManager.InterfacesRemoved'"
            .to_string(),
    ));
    dbus.call(msg_add_match)
        .await
        .expect("Could not add match rule");

    // Register the object path
    dbus.add_signal_handler("/".to_string(), sender)
        .expect("Could not register signal");
}

// This is a low level example, where the user create the channel to receive signals from specific
// peer.

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a FIFO with a size of 1024
    let (sender, mut receiver) = channel::<Message>(1024);

    register_signal(&dbus, sender.clone()).await;

    let msg = Message::method_call(
        "org.example.sender",
        "/",
        "org.freedesktop.DBus.ObjectManager",
        "GetManagedObjects",
    );

    let reply_serial = dbus
        .call_reply_serial(msg, sender)
        .await
        .expect("Could not get reply serial");
    while let Some(msg) = receiver.next().await {
        // Ignore singals until we get the MethodReturn
        if MessageType::MethodReturn == msg.get_type() {
            let msg_reply_serial = msg
                .get_reply_serial()
                .expect("Method return does not have a reply serial");
            if reply_serial == msg_reply_serial {
                // The return message of ObjectManager
                // Now we can looking for signals
                println!("GetManagedObjects: {:?}", msg);
                break;
            }
        }
    }

    // Singals are processed
    while let Some(msg) = receiver.next().await {
        println!("Signal: {:?}", msg);
    }
}
