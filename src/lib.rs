#[macro_use]
extern crate log;
#[macro_use(lazy_static)]
extern crate lazy_static;
#[macro_use(bitflags)]
extern crate bitflags;

bitflags! {
    /// An enum representing a [flag] for the `RequestName` method.
    ///
    /// [flag]: https://dbus.freedesktop.org/doc/dbus-specification.html#bus-messages-request-name
    pub struct DBusNameFlag: u32 {
        const ALLOW_REPLACEMENT = 0x01;
        const REPLACE_EXISTING = 0x02;
        const DO_NOT_QUEUE = 0x04;
    }
}

mod command;
mod connection;
mod dbus;
mod error;
mod handler;
mod introspect;
mod message;
mod server_address;
mod stream;

pub use dbus::DBus;
pub use error::{DBusError, DBusResult};
pub use handler::{Binder, Handler};
pub use server_address::{ServerAddress, ServerAddressParseError};
