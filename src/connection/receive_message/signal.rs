use super::super::connection::Connection;
use dbus_message_parser::Message;
use retain_mut::RetainMut;

impl Connection {
    pub(super) fn signal(&mut self, msg: Message) {
        // It is a Signal so we have to get the Path first.
        if let Some(path) = msg.get_path() {
            // A valid `Signal` needs to have an `Interface` and a `Member`.
            if msg.has_interface() && msg.has_member() {
                // Try to get the signal handler
                if let Some(list) = self.signals.get_mut(&path) {
                    // Go through the list and try to send the signal.
                    list.retain_mut(move |(filter, sender)| {
                        if let Some(filter) = filter {
                            if filter(&msg) {
                                return true;
                            }
                        }
                        if let Err(e) = sender.try_send(msg.clone()) {
                            if e.is_disconnected() {
                                // The handler is closed so remove it from the list.
                                return false;
                            }
                        }
                        true
                    });
                } else {
                    debug!("Signal: UNHANDLED: {:?}", msg);
                }
            }
        }
    }
}
