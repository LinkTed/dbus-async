# dbus-async
A pure Rust written asynchronous DBus library.  
[![Build status](https://github.com/LinkTed/dbus-async/workflows/Continuous%20Integration/badge.svg)](https://github.com/LinkTed/dbus-async/actions?query=workflow%3A%22Continuous+Integration%22)
[![Latest version](https://img.shields.io/crates/v/dbus-async.svg)](https://crates.io/crates/dbus-async)
[![License](https://img.shields.io/crates/l/dbus-async.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![Dependency status](https://deps.rs/repo/github/linkted/dbus-async/status.svg)](https://deps.rs/repo/github/linkted/dbus-async)

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
dbus-async = "~2.1.0"
```

You have to specify, which Tokio Runtime should be used.
* For multi-threaded add this to your `Cargo.toml`:
  ```rust
  [dependencies.tokio]
  version = " ~1.1.1"
  features = ["rt-multi-thread"] 
  ```
* For single-threaded add this to your `Cargo.toml`:
  ```rust
  [dependencies.tokio]
  version = "~1.1.1"
  features = ["rt"] 
  ```

## Example
```rust
use dbus_async::DBus;
use dbus_message_parser::Message;
use std::convert::TryInto;

#[tokio::main]
async fn main() {
    let (dbus, _server_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    // Create a MethodCall.
    let msg = Message::method_call(
        "org.freedesktop.DBus".try_into().unwrap(),
        "/org/freedesktop/DBus".try_into().unwrap(),
        "org.freedesktop.DBus.Peer".try_into().unwrap(),
        "Ping".try_into().unwrap(),
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
- [x] Implement server address [parser](https://dbus.freedesktop.org/doc/dbus-specification.html#addresses)
- [ ] Add standard interfaces:
  * [ ] [`org.freedesktop.DBus.Peer`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-peer)
  * [ ] [`org.freedesktop.DBus.ObjectManager`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager)
- [x] Add TCP support
- [ ] FD support ([PR](https://github.com/rust-lang/rust/pull/69864))
