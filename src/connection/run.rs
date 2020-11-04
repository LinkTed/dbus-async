use super::connection::Connection;
use crate::command::Command;
use futures::StreamExt;

impl Connection {
    /// Run the connection task.
    pub(crate) async fn run(mut self) {
        // Get the next command.
        while let Some(cmd) = self.command_receiver.next().await {
            match cmd {
                Command::SendMessage(msg) => self.send_message(msg),
                Command::SendMessageOneshot(msg, response) => {
                    self.send_message_oneshot(msg, response)
                }
                Command::SendMessageMpcs(msg, response_reply_serial, response) => {
                    self.send_message_mpsc(msg, response_reply_serial, response)
                }
                Command::AddMethodCall(path, object) => {
                    // Add the handler.
                    self.method_calls.insert(path, object);
                }
                Command::DeleteMethodCall(path) => {
                    // Remove the handler.
                    self.method_calls.remove(&path);
                }
                Command::DeleteMethodCallSender(sender_other) => {
                    // Remove the handler by `Sender<Message>` object.
                    self.method_calls
                        .retain(|_path, sender| !sender_other.same_receiver(sender));
                }
                Command::DeleteMethodCallReceiver(_receiver_other) => {
                    // TODO: Wait until the is_connect PR is merged:
                    // https://github.com/rust-lang/futures-rs/pull/2179
                }
                Command::ListMethodCall(path, sender) => self.list_path(&path, sender),
                Command::AddMethodCallInterface(interface, sender) => {
                    // Add an interface handler
                    self.method_calls_interface.insert(interface, sender);
                }
                Command::AddSignal(path, filter, sender) => {
                    // Add a signal handler.
                    if let Some(vec) = self.signals.get_mut(&path) {
                        vec.push((filter, sender));
                    } else {
                        self.signals.insert(path, vec![(filter, sender)]);
                    }
                }
                Command::DeleteSignalSender(sender_other) => {
                    // Remove the signal handler by `Sender<Message>` object.
                    for vec_sender_message in self.signals.values_mut() {
                        vec_sender_message
                            .retain(|(_, sender)| !sender_other.same_receiver(sender));
                    }
                }
                Command::DeleteSignalReceiver(sender_other) => {}
                Command::ReceiveMessage(msg) => self.receive_message(msg),
                Command::Close => {
                    // Stop the server.
                    return;
                }
            }
        }
    }
}
