use super::super::connection::Connection;
use dbus_message_parser::Message;

impl Connection {
    pub(super) fn method_call(&mut self, msg: Message) {
        // It is a `MethodCall` so we have to get the Path first.
        if let Some(path) = msg.get_path() {
            // A valid `MethodCall` must have a Member.
            if !msg.has_member() {
                return;
            }
            // Try to get the channel by `ObjectPath`.
            if let Some(sender) = self.method_calls.get_mut(&path) {
                let path = path.clone();
                // Try to send the `Message`.
                // This can fail if the channel is full.
                if let Err(e) = sender.try_send(msg) {
                    error!("ReceiveMessage: try to send msg: {}", path);
                    // Check if the channel is closed.
                    if e.is_disconnected() {
                        // If yes remove it from the Map.
                        error!("ReceiveMessage: object_path is disconnected: {}", path);
                        self.method_calls.remove(&path);
                    }
                    let msg = e.into_inner();
                    self.unhandled(msg);
                }
            } else if let Some(interface) = msg.get_interface() {
                // There was no such a channel by `ObjectPath`.
                // Try to find a channel by `Interface`.
                if let Some(sender) = self.method_calls_interface.get_mut(&interface) {
                    let path = path.clone();
                    let interface = interface.clone();
                    // Try to send the Message.
                    // This can fail if the channel is full.
                    if let Err(e) = sender.try_send(msg) {
                        error!("ReceiveMessage: try to send msg: {}", path);
                        // Check if the channel is closed.
                        if e.is_disconnected() {
                            // If yes remove it from the `Map`.
                            error!("ReceiveMessage: object_path is disconnected: {}", path);
                            self.method_calls_interface.remove(&interface);
                        }
                        let msg = e.into_inner();
                        self.unhandled(msg);
                    }
                } else {
                    self.unhandled(msg);
                }
            } else {
                self.unhandled(msg);
            }
        }
    }
}
