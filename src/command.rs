use dbus_message_parser::{
    match_rule::MatchRule,
    message::Message,
    value::{Interface, ObjectPath},
};
use futures::channel::{
    mpsc::{Receiver as MpscReceiver, Sender as MpscSender},
    oneshot::Sender as OneshotSender,
};
use std::collections::HashSet;

/// An enum representing all command the server task understands.
pub enum Command {
    SendMessage(Message),
    SendMessageOneshot(Message, OneshotSender<Message>),
    SendMessageMpcs(Message, OneshotSender<u32>, MpscSender<Message>),
    AddMethodCall(ObjectPath, MpscSender<Message>),
    DeleteMethodCall(ObjectPath),
    DeleteMethodCallSender(MpscSender<Message>),
    DeleteMethodCallReceiver(MpscReceiver<Message>),
    ListMethodCall(ObjectPath, OneshotSender<HashSet<String>>),
    AddMethodCallInterface(Interface, MpscSender<Message>),
    DeleteMethodCallInterface(Interface),
    DeleteMethodCallInterfaceSender(MpscSender<Message>),
    DeleteMethodCallInterfaceReceiver(MpscReceiver<Message>),
    AddSignal(
        ObjectPath,
        Option<fn(&Message) -> bool>,
        MpscSender<Message>,
    ),
    DeleteSignalSender(MpscSender<Message>),
    DeleteSignalReceiver(MpscReceiver<Message>),
    AddMatchRules(Vec<MatchRule>, MpscSender<Message>),
    DeleteMatchRulesSender(MpscSender<Message>),
    DeleteMatchRulesReceiver(MpscReceiver<Message>),
    Close,
}
