use dbus_message_parser::Message;
use futures::channel::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use futures::channel::oneshot::Sender as OneshotSender;
use std::collections::HashSet;

/// An enum representing all command the server task understands.
pub enum Command {
    SendMessage(Message),
    SendMessageOneshot(Message, OneshotSender<Message>),
    SendMessageMpcs(Message, OneshotSender<u32>, MpscSender<Message>),
    AddPath(String, MpscSender<Message>),
    DeletePath(String),
    DeleteSender(MpscSender<Message>),
    DeleteReceiver(MpscReceiver<Message>),
    ListPath(String, OneshotSender<HashSet<String>>),
    AddInterface(String, MpscSender<Message>),
    AddSignalHandler(String, Option<fn(&Message) -> bool>, MpscSender<Message>),
    DeleteSignalHandler(MpscSender<Message>),
    ReceiveMessage(Message),
    Close,
}
