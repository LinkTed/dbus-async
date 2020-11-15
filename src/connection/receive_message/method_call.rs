use super::super::connection::Connection;
use dbus_message_parser::Message;

impl Connection {
    /// Try to find a sender by `ObjectPath`.
    /// If there was no sender founded then it will return the given message back.
    #[inline]
    fn find_sender_by_object_path(&mut self, msg: Message) -> Option<Message> {
        let object_path = msg.get_path().unwrap();
        // Try to get the channel by `ObjectPath`.
        if let Some(sender) = self.method_calls.get_mut(&object_path) {
            let object_path = object_path.clone();
            // Try to send the `Message`.
            // This can fail if the channel is full.
            match sender.try_send(msg) {
                Ok(()) => None,
                Err(e) => {
                    error!("ReceiveMessage: try to send msg: {}", object_path);
                    let is_disconnected = e.is_disconnected();
                    let msg = e.into_inner();
                    // Check if the channel is closed.
                    if is_disconnected {
                        // If yes remove it from the Map.
                        error!(
                            "ReceiveMessage: object_path is disconnected: {}",
                            object_path
                        );
                        self.method_calls.remove(&object_path);
                        // INFO: Next, try to find a sender by `Interface`.
                        Some(msg)
                    } else {
                        self.unhandled(msg);
                        None
                    }
                }
            }
        } else {
            Some(msg)
        }
    }

    /// Try to find a sender by `Interface`.
    /// If there was no sender founded then it will return the given message back.
    #[inline]
    fn find_sender_by_interface(&mut self, msg: Message) -> Option<Message> {
        if let Some(interface) = msg.get_interface() {
            // Try to get the channel by `Interface`.
            if let Some(sender) = self.method_calls_interface.get_mut(interface) {
                let interface = interface.clone();
                // Try to send the Message.
                // This can fail if the channel is full.
                match sender.try_send(msg) {
                    Ok(()) => None,
                    Err(e) => {
                        error!("ReceiveMessage: try to send msg: {}", interface);
                        // Check if the channel is closed.
                        if e.is_disconnected() {
                            // If yes remove it from the `Map`.
                            error!("ReceiveMessage: interface is disconnected: {}", interface);
                            self.method_calls_interface.remove(&interface);
                        }
                        let msg = e.into_inner();
                        Some(msg)
                    }
                }
            } else {
                Some(msg)
            }
        } else {
            Some(msg)
        }
    }

    pub(super) fn method_call(&mut self, msg: Message) {
        // Try to find a sender for this message by `ObjectPath`.
        let msg = self.find_sender_by_object_path(msg);
        if let Some(msg) = msg {
            // If there was no sender founded then try to find a sender for this message by
            // `Interface`.
            if let Some(msg) = self.find_sender_by_interface(msg) {
                self.unhandled(msg);
            }
        }
    }
}
