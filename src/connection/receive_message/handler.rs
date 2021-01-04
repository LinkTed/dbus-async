use super::super::connection::Connection;
use dbus_message_parser::message::{Message, MessageType};

impl Connection {
    pub(in super::super) fn receive_message(&mut self, msg: Message) {
        // Receive a Message
        match msg.get_type() {
            MessageType::MethodCall => self.method_call(msg),
            MessageType::MethodReturn => self.method_return(msg),
            MessageType::Error => self.error(msg),
            MessageType::Signal => self.signal(msg),
        }
    }
}
