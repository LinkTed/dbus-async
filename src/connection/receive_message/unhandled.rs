use super::super::connection::Connection;
use dbus_message_parser::message::Message;

impl Connection {
    pub(super) fn unhandled(&mut self, msg: Message) {
        error!("MethodCall: UNHANDLED: {:?}", msg);
        if let Some(mut msg) = msg.unknown_path() {
            self.serial += 1;
            msg.set_serial(self.serial);

            if let Err(e) = self.message_sender.unbounded_send(msg) {
                error!("MethodCall: message_sender.unbounded_send: {:?}", e);
            }
        }
    }
}
