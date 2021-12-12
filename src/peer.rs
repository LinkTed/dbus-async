use crate::{DBus, DBusResult, Uuid};
use dbus_message_parser::{
    message::{Message, MessageHeader, MessageType},
    value::Value,
};
use futures::{
    channel::mpsc::{channel, Receiver},
    StreamExt,
};
use hex::{decode_to_slice, encode, FromHexError};
use std::{
    convert::TryInto,
    io::Error as IoError,
    str::{from_utf8, Utf8Error},
    vec::IntoIter,
};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt, spawn};

#[derive(Debug, Error)]
enum MachineIdError {
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
    #[error("Machine ID file is not UTF-8: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Machine ID is too large")]
    FileTooLarge,
    #[error("Machine ID is too small")]
    FileTooSmall,
    #[error("Could not decode hex string")]
    FromHexError(#[from] FromHexError),
}

async fn read_machine_id_from_file(path: &str) -> Result<Uuid, MachineIdError> {
    let mut file = File::open(path).await?;
    let mut uuid_str: [u8; 32] = [0; 32];
    let mut uuid: Uuid = [0; 16];
    let mut new_line: [u8; 2] = [0; 2];

    let read = file.read_exact(&mut uuid_str).await?;
    if read == uuid_str.len() {
        // Check if there is more bytes
        let read = file.read(&mut new_line).await?;
        let uuid_str = from_utf8(&uuid_str[..])?;
        if (read == 0 || read == 1) && (new_line[0] == 0 || new_line[0] == b'\n') {
            decode_to_slice(uuid_str, &mut uuid[..])?;
            Ok(uuid)
        } else {
            Err(MachineIdError::FileTooLarge)
        }
    } else {
        Err(MachineIdError::FileTooSmall)
    }
}

async fn get_machine_id_from_file() -> Result<Uuid, ()> {
    match read_machine_id_from_file("/var/lib/dbus/machine-id").await {
        Ok(uuid) => Ok(uuid),
        Err(e) => {
            error!(
                "Could not read Machine ID from /var/lib/dbus/machine-id: {}",
                e
            );
            // Fallback to /etc/machine-id.
            match read_machine_id_from_file("/etc/machine-id").await {
                Ok(uuid) => Ok(uuid),
                Err(e) => {
                    error!("Could not read Machine ID from /etc/machine-id: {}", e);
                    Err(())
                }
            }
        }
    }
}

async fn get_machine_id(header: &MessageHeader) -> Message {
    match header.method_return() {
        Ok(mut msg) => match get_machine_id_from_file().await {
            Ok(uuid) => {
                let uuid = encode(&uuid);
                msg.add_value(Value::String(uuid));
                msg
            }
            Err(_) => header.error(
                "org.freedesktop.DBus.Peer.MachineIdError"
                    .try_into()
                    .unwrap(),
                "Could not retrieve Machine ID.".to_string(),
            ),
        },
        Err(msg) => msg,
    }
}

/// This is the handle method for the [`org.freedesktop.DBus.Peer`] interface.
///
/// [`org.freedesktop.DBus.Peer`]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-peer
pub async fn handle_peer(
    dbus: &DBus,
    header: MessageHeader,
    mut body_iter: IntoIter<Value>,
) -> DBusResult<()> {
    let member = if let Some(member) = header.get_member() {
        member
    } else {
        return Ok(());
    };

    let response = match member.as_ref() {
        "GetMachineId" => {
            if body_iter.next().is_none() {
                get_machine_id(&header).await
            } else {
                header.invalid_args("Too many arguments".to_string())
            }
        }
        "Ping" => {
            if body_iter.next().is_none() {
                // The unwrap function call will never panic because we check the type at the
                // beginning of the while loop.
                header.method_return().unwrap()
            } else {
                header.invalid_args("Too many arguments".to_string())
            }
        }
        _ => {
            if let Some(msg) = header.unknown_member() {
                msg
            } else {
                return Ok(());
            }
        }
    };

    dbus.send(response)
}

async fn peer(dbus: DBus, mut receiver: Receiver<Message>) {
    while let Some(request) = receiver.next().await {
        if MessageType::MethodCall != request.get_type() {
            continue;
        }

        if let Ok((header, body)) = request.split() {
            if let Err(e) = handle_peer(&dbus, header, body.into_iter()).await {
                error!(
                    "Could not handle method for org.freedesktop.DBus.Peer: {}",
                    e
                );
            }
        }
    }
}

pub(super) fn add_peer(dbus: DBus) -> DBusResult<()> {
    let (sender, receiver) = channel(1024);
    let interface = "org.freedesktop.DBus.Peer".try_into().unwrap();
    // Try to add the interface handler.
    if let Err(e) = dbus.add_method_call_interface(interface, sender) {
        return Err(e);
    }

    spawn(peer(dbus, receiver));
    Ok(())
}
