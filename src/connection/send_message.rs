use super::connection::Connection;
use dbus_message_parser::Message;
use futures::channel::oneshot::Sender;

impl Connection {
    pub(super) fn send_message(&mut self, mut msg: Message, response: Option<Sender<Message>>) {
        // Get the sender, if one is present.
        if let Some(sender) = msg.get_sender() {
            // Get the destination, if one is present.
            if let Some(destination) = msg.get_destination() {
                // Check if the sender and destination are the same.
                if sender == destination {
                    // Sender and destination should not be the same.
                    error!("sender == destination");
                    return;
                }
            }
        }
        // Increment the serial number.
        self.serial += 1;
        msg.set_serial(self.serial);
        // Add the response handler to the Map.
        // This means that the user wants the response.
        if let Some(sender) = response {
            self.replies.put(self.serial, sender);
        }
        // Send the message.
        if let Err(e) = self.message_sender.unbounded_send(msg) {
            error!("Connection: message_sender.unbounded_send: {:?}", e);
            return;
        }
    }
}
