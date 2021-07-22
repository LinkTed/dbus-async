#[macro_use]
extern crate log;
#[macro_use(bitflags)]
extern crate bitflags;

mod command;
mod connection;
mod dbus;
mod error;
mod handler;
mod introspect;
mod name_flag;
mod peer;
mod stream;

type Uuid = [u8; 16];

pub use dbus::DBus;
pub use error::{DBusError, DBusResult};
pub use handler::{Binder, Handler};
pub use name_flag::DBusNameFlag;
pub use peer::handle_peer;
