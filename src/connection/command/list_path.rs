use super::super::Connection;
use dbus_message_parser::value::ObjectPath;
use futures::channel::oneshot::Sender;
use std::collections::HashSet;

impl Connection {
    pub(super) fn list_path(&mut self, object_path: &ObjectPath, sender: Sender<HashSet<String>>) {
        // List the handler.
        let mut result = HashSet::new();

        for p in self.method_calls.keys() {
            if let Some(mut split) = p.strip_prefix_elements(object_path) {
                if let Some(base) = split.next() {
                    result.insert(base.to_string());
                }
            }
        }

        if let Err(e) = sender.send(result) {
            error!("ListPath: cannot send result: {:?}", e);
        }
    }
}
