use super::Connection;
use futures::StreamExt;

impl Connection {
    fn has_channels(&self) -> bool {
        !self.signals.is_empty()
            || !self.method_calls.is_empty()
            || !self.method_calls_interface.is_empty()
    }

    async fn receive_only_message(&mut self) {
        if self.has_channels() {
            while let Some(msg) = self.message_stream.next().await {
                self.receive_message(msg);
                if !self.has_channels() {
                    debug!("Has not channels");
                    break;
                }
            }
        }
        debug!("Message stream is closed");
    }

    /// Run the connection task.
    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                next = self.message_stream.next() => match next {
                    Some(msg) => self.receive_message(msg),
                    None => {
                        debug!("Message stream is closed");
                        break;
                    }
                },
                // Get the next command.
                next = self.command_receiver.next() => match next {
                    Some(cmd) => self.receive_command(cmd),
                    None => {
                        debug!("Command stream is closed");
                        self.receive_only_message().await;
                        break;
                    }
                },
                else => {
                    debug!("Both stream are closed");
                    break;
                }
            }
        }
    }
}
