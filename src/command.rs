use dbus_message_parser::Message;
use futures::channel::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use futures::channel::oneshot::Sender as OneshotSender;
use std::collections::HashSet;

/// An enum representing all command the server task understands.
#[derive(Debug)]
pub enum Command {
    SendMessage(Message, Option<OneshotSender<Message>>),
    AddPath(String, MpscSender<Message>),
    DeletePath(String),
    DeleteSender(MpscSender<Message>),
    DeleteReceiver(MpscReceiver<Message>),
    ListPath(String, OneshotSender<HashSet<String>>),
    AddInterface(String, MpscSender<Message>),
    AddSignalHandler(String, MpscSender<Message>),
    DeleteSignalHandler(MpscSender<Message>),
    ReceiveMessage(Message),
    Close,
}
