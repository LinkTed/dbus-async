use crate::command::Command;
use dbus_message_parser::Message;
use futures::channel::mpsc::{Sender as MpscSender, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot::Sender as OneshotSender;
use lru::LruCache;
use std::collections::HashMap;

pub(crate) struct Connection {
    pub(super) serial: u32,
    pub(super) replies: LruCache<u32, OneshotSender<Message>>,
    pub(super) signals: HashMap<String, Vec<MpscSender<Message>>>,
    pub(super) path_handler: HashMap<String, MpscSender<Message>>,
    pub(super) interface_handler: HashMap<String, MpscSender<Message>>,
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
            path_handler: HashMap::new(),
            interface_handler: HashMap::new(),
            command_receiver,
            message_sender,
        }
    }
}
