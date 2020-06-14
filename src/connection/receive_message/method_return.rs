use super::super::connection::Connection;
use dbus_message_parser::Message;

impl Connection {
    pub(super) fn method_return(&mut self, msg: Message) {
        // It is a MethodCall so we have to get the reply
        // serial if there is one.
        if let Some(serial) = msg.get_reply_serial() {
            // Try to get the response handler.
            if let Some(sender) = self.replies.pop(&serial) {
                // Try to send it.
                if let Err(e) = sender.send(msg) {
                    error!("MethodReturn: sender.send: {:?}", e);
                }
            } else {
                debug!("MethodReturn: UNHANDLED: {:?}", msg);
            }
        }
    }
}
