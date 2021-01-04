use std::net::SocketAddr;

/// This represents a DBus [client address] (connectable address).
///
/// [client address]: https://dbus.freedesktop.org/doc/dbus-specification.html#addresses
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientAddress {
    UnixPath(String),
    UnixAbstract(String),
    Tcp(SocketAddr),
}
