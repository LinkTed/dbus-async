use dbus_async::{DBus, DBusNameFlag};
use std::convert::TryInto;

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Register name
    let result = dbus
        .register_name(
            "org.example.DBus".try_into().unwrap(),
            &DBusNameFlag::empty(),
        )
        .await;

    // Print if it was successful
    println!("{:?}", result);
}
