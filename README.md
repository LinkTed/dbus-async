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
dbus-async = "~2.3.0"
```

You have to specify, which Tokio Runtime should be used.
* For multi-threaded add this to your `Cargo.toml`:
  ```rust
  [dependencies.tokio]
  version = " ~1.15.0"
  features = ["rt-multi-thread"]
  ```
* For single-threaded add this to your `Cargo.toml`:
  ```rust
  [dependencies.tokio]
  version = "~1.15.0"
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

## Features
- [Authentication Protocol](https://dbus.freedesktop.org/doc/dbus-specification.html#auth-protocol)
  * [x] [`EXTERNAL`](https://dbus.freedesktop.org/doc/dbus-specification.html#auth-mechanisms-external)
  * [x] [`ANONYMOUS`](https://dbus.freedesktop.org/doc/dbus-specification.html#auth-mechanisms-anonymous)
  * [ ] [`DBUS_COOKIE_SHA1`](https://dbus.freedesktop.org/doc/dbus-specification.html#auth-mechanisms-sha)
- [Server Addresses](https://dbus.freedesktop.org/doc/dbus-specification.html#addresses)
  * [ ] [`unix`](https://dbus.freedesktop.org/doc/dbus-specification.html#transports-unix-domain-sockets-addresses)
    - [x] `path`
    - [ ] `abstract`
  * [x] [`unixexec`](https://dbus.freedesktop.org/doc/dbus-specification.html#transports-exec)
        (argv0 is not supported)
  * [x] [`tcp`](https://dbus.freedesktop.org/doc/dbus-specification.html#transports-tcp-sockets)
  * [x] [`nonce-tcp`](https://dbus.freedesktop.org/doc/dbus-specification.html#transports-nonce-tcp-sockets)
  * [ ] [`launchd`](https://dbus.freedesktop.org/doc/dbus-specification.html#transports-launchd)
  * [ ] [`autolaunch`](https://dbus.freedesktop.org/doc/dbus-specification.html#meta-transports-autolaunch)
- [Standard Interfaces](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces)
  * [x] [`org.freedesktop.DBus.Introspectable`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-introspectable)
  * [x] [`org.freedesktop.DBus.Peer`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-peer)
  * [ ] [`org.freedesktop.DBus.ObjectManager`](https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager)
- [ ] FD support ([Tracking Issue](https://github.com/rust-lang/rust/issues/76915))
