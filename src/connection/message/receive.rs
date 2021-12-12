use super::super::Connection;
use dbus_message_parser::{
    match_rule::MatchRule,
    message::{Message, MessageType},
};

impl Connection {
    pub(in super::super) fn receive_message(&mut self, msg: Message) {
        for (match_rules, sender) in self.match_rules.iter_mut() {
            if MatchRule::matching_rules(match_rules, &msg) {
                if let Err(e) = sender.try_send(msg.clone()) {
                    error!("mpsc.try_send: {:?}", e);
                }
            }
        }

        match msg.get_type() {
            MessageType::MethodCall => self.method_call(msg),
            MessageType::MethodReturn => self.method_return(msg),
            MessageType::Error => self.error(msg),
            MessageType::Signal => self.signal(msg),
        }
    }
}
