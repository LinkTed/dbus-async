use crate::command::Command;
use dbus_message_parser::Message;
use futures::channel::mpsc::TrySendError;
use futures::channel::oneshot::Canceled;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

#[derive(Debug)]
pub enum DBusError {
    SendMessage(Message),
    AddPath(String),
    DeletePath(Option<String>),
    ListPath(String),
    AddInterface(String),
    AddSignalHandler(String),
    DeleteSignalHandler,
    RecvMessage(Option<Message>),
    Close,
}

impl From<TrySendError<Command>> for DBusError {
    fn from(e: TrySendError<Command>) -> Self {
        match e.into_inner() {
            Command::SendMessage(msg, _) => DBusError::SendMessage(msg),
            Command::AddPath(object_path, _) => DBusError::AddPath(object_path),
            Command::DeletePath(object_path) => DBusError::DeletePath(Some(object_path)),
            Command::DeleteSender(_) => DBusError::DeletePath(None),
            Command::DeleteReceiver(_) => DBusError::DeletePath(None),
            Command::ListPath(object_path, _) => DBusError::ListPath(object_path),
            Command::AddInterface(object_path, _) => DBusError::AddInterface(object_path),
            Command::AddSignalHandler(object_path, _) => DBusError::AddSignalHandler(object_path),
            Command::DeleteSignalHandler(_) => DBusError::DeleteSignalHandler,
            Command::ReceiveMessage(msg) => DBusError::RecvMessage(Some(msg)),
            Command::Close => DBusError::Close,
        }
    }
}

impl From<Canceled> for DBusError {
    fn from(_: Canceled) -> Self {
        DBusError::RecvMessage(None)
    }
}

impl From<DBusError> for IoError {
    fn from(e: DBusError) -> Self {
        IoError::new(IoErrorKind::Other, format!("call_hello: {:?}", e))
    }
}

impl Display for DBusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DBusError::SendMessage(msg) => write!(f, "Could not send message: {:?}", msg),
            DBusError::AddPath(path) => write!(f, "Could not add object to path: {}", path),
            DBusError::DeletePath(_object_path) => write!(f, "Could not delete object from path"),
            DBusError::ListPath(path) => write!(f, "Could not list object of path: {}", path),
            DBusError::AddInterface(interface) => {
                write!(f, "Could not add object to interface: {}", interface)
            }
            DBusError::AddSignalHandler(path) => {
                write!(f, "Could not add signal handler for path: {}", path)
            }
            DBusError::DeleteSignalHandler => write!(f, "Could not delete signal handler"),
            DBusError::RecvMessage(msg) => {
                write!(f, "Could not receive response for message: {:?}", msg)
            }
            DBusError::Close => write!(f, "Could not close DBus"),
        }
    }
}

pub type DBusResult<T> = std::result::Result<T, DBusError>;
