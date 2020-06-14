# dbus-async
A pure Rust written asynchronous DBus library.

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
dbus-async = "1.0"
```

## Example
```rust
use dbus_async::DBus;
use dbus_message_parser::Message;

#[tokio::main]
async fn main() {
    let (dbus, _server_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall.
    let msg = Message::method_call(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.Peer",
        "Ping",
    );

    // Send the message and get the return message.
    let return_msg = dbus.call(msg).await;

    // Print the return message.
    println!("{:?}", return_msg);
}
```
If you want to implement a DBus service and do not implement the `dbus_async::Handler` trait 
manually then use `dbus-async-derive` crate.

## TODO
- [ ] Implement server address [parser](https://dbus.freedesktop.org/doc/dbus-specification.html#addresses)
- [ ] Add standard interfaces:
  * [ ] [`org.freedesktop.DBus.Peer`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-peer)
  * [ ] [`org.freedesktop.DBus.ObjectManager`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager)
- [ ] Add TCP support
- [ ] FD support ([PR](https://github.com/rust-lang/rust/pull/69864))
