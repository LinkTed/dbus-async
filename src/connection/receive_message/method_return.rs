use super::super::connection::{Connection, MessageSender};
use dbus_message_parser::Message;

impl Connection {
    pub(super) fn method_return(&mut self, msg: Message) {
        // It is a MethodCall so we have to get the reply
        // serial if there is one.
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
            debug!("UNHANDLED: {:?}", msg);
        }
    }
}
