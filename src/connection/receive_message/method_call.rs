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
            let path = path.to_string();
            // Try to get the handler by `Path`.
            if let Some(sender) = self.path_handler.get_mut(&path) {
                let path = path.clone();
                // Try to send the `Message`.
                // This can fail if the handler is full.
                if let Err(e) = sender.try_send(msg) {
                    error!("ReceiveMessage: try to send msg: {}", path);
                    // Check if the handler is closed.
                    if e.is_disconnected() {
                        // If yes remove it from the Map.
                        error!("ReceiveMessage: object_path is disconnected: {}", path);
                        self.path_handler.remove(&path);
                    }
                    let msg = e.into_inner();
                    self.unhandled(msg);
                }
            } else if let Some(interface) = msg.get_interface() {
                // There was no such a handler by `Path`.
                // Try to find a handler by `Interface`.
                // Get the interface of the `Message`.
                let interface = interface.to_string();
                // Try to get the handler by Interface
                if let Some(sender) = self.interface_handler.get_mut(&interface) {
                    let path = path.clone();
                    // Try to send the Message.
                    // This can fail if the handler is full.
                    if let Err(e) = sender.try_send(msg) {
                        error!("ReceiveMessage: try to send msg: {}", path);
                        // Check if the handler is closed.
                        if e.is_disconnected() {
                            // If yes remove it from the `Map`.
                            error!("ReceiveMessage: object_path is disconnected: {}", path);
                            self.interface_handler.remove(&interface);
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
