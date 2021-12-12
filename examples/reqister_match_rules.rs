use dbus_async::DBus;
use dbus_message_parser::{
    match_rule::MatchRule,
    message::{Message, MessageType},
    value::Value,
};
use futures::{channel::mpsc::channel, stream::StreamExt};
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
    let match_rules = vec![
        MatchRule::Type(MessageType::Signal),
        //MatchRule::Sender("org.freedesktop.DBus".try_into().unwrap()),
        //MatchRule::Path("/org/example/DBus".try_into().unwrap()),
        //MatchRule::Interface("org.freedesktop.DBus".try_into().unwrap()),
    ];
    println!("{}", MatchRule::encode(&match_rules));
    msg_add_match.add_value(Value::String(MatchRule::encode(&match_rules)));
    dbus.call(msg_add_match)
        .await
        .expect("Could not add match rule");

    // Initialize the match rules to receive only signals from org.freedesktop.DBus
    let match_rules = vec![
        MatchRule::Type(MessageType::Signal),
        MatchRule::Sender("org.freedesktop.DBus".try_into().unwrap()),
    ];

    // Create a FIFO with a size of 1024
    let (sender, mut receiver) = channel::<Message>(1024);

    // Register the object path
    if let Err(e) = dbus.add_match_rules(match_rules, sender) {
        panic!("Cannot add path: {:?}", e);
    }

    // Get the any signal from the DBus
    while let Some(msg) = receiver.next().await {
        println!("{:?}", msg);
    }
}
