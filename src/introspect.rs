use crate::{DBus, DBusResult};
use dbus_message_parser::{message::Message, value::Value};
use futures::{
    channel::mpsc::{channel, Receiver},
    StreamExt,
};
use std::convert::TryInto;
use tokio::spawn;

const XML: &str = "<!DOCTYPE node PUBLIC \"-//freedesktop//DTD \
    D-BUS Object Introspection 1.0//EN\" \"http://www.freedesktop.org/\
    standards/dbus/1.0/introspect.dtd\">\n<node>\n";

async fn introspect(dbus: DBus, mut receiver: Receiver<Message>) {
    while let Some(msg) = receiver.next().await {
        let member = if let Some(member) = msg.get_member() {
            member
        } else {
            continue;
        };
        // Check the member.
        match member.as_ref() {
            "Introspect" => {
                // Check if the signature of the message is correct.
                if !msg.get_signature().unwrap().is_empty() {
                    let msg = msg.invalid_args("Too many arguments".to_string());
                    if let Err(e) = dbus.send(msg) {
                        error!("could not send message: {}", e);
                        return;
                    }
                    continue;
                }
                // Get the path for which another peer wants to introspect.
                if let Some(path) = msg.get_path() {
                    match dbus.list_method_call(path.clone()).await {
                        Ok(list) => {
                            // Create a return message.
                            let msg = match msg.method_return() {
                                Ok(mut msg) => {
                                    // Create the XML response.
                                    // Start with the header.
                                    let mut xml = XML.to_string();
                                    // Add all nodes to the XML body.
                                    for l in list {
                                        xml += &format!("  <node name=\"{}\"/>\n", l);
                                    }
                                    xml += "</node>";
                                    // Add the return value.
                                    // Send the return message.
                                    msg.add_value(Value::String(xml));
                                    msg
                                }
                                Err(msg) => msg,
                            };
                            if let Err(e) = dbus.send(msg) {
                                error!("could not send message: {}", e);
                                return;
                            }
                        }
                        Err(e) => {
                            error!("Introspect: {:?}", e);
                        }
                    }
                }
            }
            _ => {
                if let Some(msg) = msg.unknown_member() {
                    if let Err(e) = dbus.send(msg) {
                        error!("could not send message: {}", e);
                        return;
                    }
                }
            }
        }
    }
}

pub(super) fn add_introspect(dbus: DBus) -> DBusResult<()> {
    // If introspectable is true then add the introspectable interface handler.
    let (sender, receiver) = channel(1024);
    let interface = "org.freedesktop.DBus.Introspectable".try_into().unwrap();
    // Try to add the interface handler.
    if let Err(e) = dbus.add_method_call_interface(interface, sender) {
        return Err(e);
    }

    // Spawn the introspectable handler.
    spawn(introspect(dbus, receiver));
    Ok(())
}
