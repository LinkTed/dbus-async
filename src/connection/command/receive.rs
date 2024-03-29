use super::super::Connection;
use crate::command::Command;

impl Connection {
    pub(in super::super) fn receive_command(&mut self, cmd: Command) {
        match cmd {
            Command::SendMessage(msg) => self.send_message(msg),
            Command::SendMessageOneshot(msg, response) => self.send_message_oneshot(msg, response),
            Command::SendMessageMpcs(msg, response_reply_serial, response) => {
                self.send_message_mpsc(msg, response_reply_serial, response)
            }
            Command::AddMethodCall(object_path, object) => {
                // Add the handler.
                self.method_calls.insert(object_path, object);
            }
            Command::DeleteMethodCall(object_path) => {
                // Remove the handler.
                self.method_calls.remove(&object_path);
            }
            Command::DeleteMethodCallSender(sender_other) => {
                // Remove the handler by `Sender<Message>` object.
                self.method_calls
                    .retain(|_, sender| !sender_other.same_receiver(sender));
            }
            Command::DeleteMethodCallReceiver(receiver) => {
                self.method_calls
                    .retain(|_, sender| !sender.is_connected_to(&receiver));
            }
            Command::ListMethodCall(object_path, sender) => self.list_path(&object_path, sender),
            Command::AddMethodCallInterface(interface, sender) => {
                // Add an interface handler
                self.method_calls_interface.insert(interface, sender);
            }
            Command::DeleteMethodCallInterface(interface) => {
                self.method_calls_interface.remove(&interface);
            }
            Command::DeleteMethodCallInterfaceSender(sender_other) => {
                // Remove the handler by `Sender<Message>` object.
                self.method_calls_interface
                    .retain(|_, sender| !sender_other.same_receiver(sender));
            }
            Command::DeleteMethodCallInterfaceReceiver(receiver) => {
                // Remove the handler by `Sender<Message>` object.
                self.method_calls_interface
                    .retain(|_, sender| !sender.is_connected_to(&receiver));
            }
            Command::AddSignal(object_path, filter, sender) => {
                // Add a signal handler.
                if let Some(vec) = self.signals.get_mut(&object_path) {
                    vec.push((filter, sender));
                } else {
                    self.signals.insert(object_path, vec![(filter, sender)]);
                }
            }
            Command::DeleteSignalSender(sender_other) => {
                // Remove the signal handler by `Sender<Message>` object.
                for vec_sender_message in self.signals.values_mut() {
                    vec_sender_message.retain(|(_, sender)| !sender_other.same_receiver(sender));
                }
            }
            Command::DeleteSignalReceiver(receiver) => {
                for vec_sender_message in self.signals.values_mut() {
                    vec_sender_message.retain(|(_, sender)| !sender.is_connected_to(&receiver));
                }
            }
            Command::AddMatchRules(match_rules, sender) => {
                self.match_rules.push((match_rules, sender));
            }
            Command::DeleteMatchRulesSender(sender_other) => {
                self.match_rules
                    .retain(|(_, sender)| !sender_other.same_receiver(sender));
            }
            Command::DeleteMatchRulesReceiver(receiver) => {
                self.match_rules
                    .retain(|(_, sender)| !sender.is_connected_to(&receiver));
            }
            Command::Close => {
                // Stop the server.
                self.command_receiver.close();
                self.message_stream.close();
                self.message_sink.close_channel();
                self.method_calls.clear();
                self.method_calls_interface.clear();
                self.signals.clear();
            }
        }
    }
}
