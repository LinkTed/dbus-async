use super::super::{Connection, MessageSender};
use dbus_message_parser::message::Message;

impl Connection {
    pub(super) fn error(&mut self, msg: Message) {
        // It is an Error so we have to get the reply serial
        let serial = msg.get_reply_serial().unwrap();
        // Try to get the response handler.
        if let Some(sender) = self.replies.pop(&serial) {
            // Try to send it.
            match sender {
                MessageSender::Oneshot(sender) => {
                    if let Err(e) = sender.send(msg) {
                        error!("oneshot.send: {:?}", e);
                    }
                }
                MessageSender::Mpcs(mut sender) => {
                    if let Err(e) = sender.try_send(msg) {
                        error!("mpsc.try_send: {:?}", e);
                    }
                }
            }
        } else {
            debug!("Error: UNHANDLED: {:?}", msg);
        }
    }
}
