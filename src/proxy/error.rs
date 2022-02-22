use crate::DBusError;
use dbus_message_parser::{
    message::Message,
    value::{BusError, InterfaceError, MemberError, ObjectPathError, Type, TypeError},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    DBusError(#[from] DBusError),
    #[error(transparent)]
    InterfaceError(#[from] InterfaceError),
    #[error(transparent)]
    ObjectPathError(#[from] ObjectPathError),
    #[error(transparent)]
    MemberError(#[from] MemberError),
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    BusError(#[from] BusError),
    #[error("{0}")]
    FromPeer(String),
    #[error("unexpected data type {0:?}")]
    UnexpectedFormat(Vec<Type>),
}

impl From<Message> for Error {
    fn from(msg: Message) -> Self {
        if let Some(err) = msg.get_error_name() {
            return Error::FromPeer(format!("{} {:?}", err, msg.get_body()));
        }
        return Error::FromPeer(format!("{:?}", msg.get_body()));
    }
}
