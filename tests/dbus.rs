use dbus_async::{DBus, DBusNameFlag};
use dbus_message_parser::message::Message;
use dbus_message_parser::value::Value;
use std::convert::TryInto;

#[tokio::test]
async fn message_send() {
    let (dbus, connection_handle) = DBus::session(true)
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
    dbus.send(msg).unwrap();

    // Close and wait until the message is really sent.
    dbus.close().unwrap();
    connection_handle.await.unwrap();
}

#[tokio::test]
async fn method_call() {
    let (dbus, _connection_handle) = DBus::session(true)
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
    dbus.call(msg).await.unwrap();
}

#[tokio::test]
async fn method_call_with_args() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall
    let mut msg = Message::method_call(
        "org.freedesktop.DBus".try_into().unwrap(),
        "/org/freedesktop/DBus".try_into().unwrap(),
        "org.freedesktop.DBus".try_into().unwrap(),
        "AddMatch".try_into().unwrap(),
    );

    // Add value as argument
    msg.add_value(Value::String(
        "type='signal',sender='org.freedesktop.DBus'".to_string(),
    ));

    // Send the message and get the return message
    dbus.call(msg).await.unwrap();
}

#[tokio::test]
async fn request_name() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Register name
    dbus.request_name(
        "org.example.DBus".try_into().unwrap(),
        &DBusNameFlag::empty(),
    )
    .await
    .unwrap();
}
