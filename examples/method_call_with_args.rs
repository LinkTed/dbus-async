use dbus_async::DBus;
use dbus_message_parser::{Message, Value};

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall
    let mut msg = Message::method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "AddMatch",
    );

    // Add value as argument
    msg.add_value(Value::String(
        "type='signal',sender='org.freedesktop.DBus'".to_string(),
    ));

    // Send the message and get the return message
    let return_msg = dbus.call(msg).await;

    // Print the return message
    println!("{:?}", return_msg);
}
