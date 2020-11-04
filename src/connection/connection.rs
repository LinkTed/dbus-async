use crate::command::Command;
use dbus_message_parser::{Interface, Message, ObjectPath};
use futures::channel::mpsc::{Sender as MpscSender, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot::Sender as OneshotSender;
use lru::LruCache;
use std::collections::HashMap;

pub(crate) enum MessageSender {
    Oneshot(OneshotSender<Message>),
    Mpcs(MpscSender<Message>),
}

pub(crate) struct Connection {
    pub(super) serial: u32,
    pub(super) replies: LruCache<u32, MessageSender>,
    pub(super) signals:
        HashMap<ObjectPath, Vec<(Option<fn(&Message) -> bool>, MpscSender<Message>)>>,
    pub(super) method_calls: HashMap<ObjectPath, MpscSender<Message>>,
    pub(super) method_calls_interface: HashMap<Interface, MpscSender<Message>>,
    pub(super) command_receiver: UnboundedReceiver<Command>,
    pub(super) message_sender: UnboundedSender<Message>,
}

impl Connection {
    pub(crate) fn from(
        command_receiver: UnboundedReceiver<Command>,
        message_sender: UnboundedSender<Message>,
    ) -> Connection {
        Connection {
            serial: 0,
            replies: LruCache::new(1024),
            signals: HashMap::new(),
            method_calls: HashMap::new(),
            method_calls_interface: HashMap::new(),
            command_receiver,
            message_sender,
        }
    }
}
