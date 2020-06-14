use super::connection::Connection;
use futures::channel::oneshot::Sender;
use std::collections::HashSet;
use std::ops::Deref;

impl Connection {
    pub(super) fn list_path(&mut self, path: &String, sender: Sender<HashSet<String>>) {
        // List the handler.
        let mut result = HashSet::new();

        let path_with_slash = if path == "/" {
            path.clone()
        } else {
            format!("{}/", path)
        };

        let split_at = path_with_slash.len();
        for p in self.path_handler.keys() {
            if p.deref() == path {
                continue;
            } else if p.starts_with(&path_with_slash) {
                let (_, last) = p.split_at(split_at);
                let base: Vec<&str> = last.splitn(2, "/").collect();
                if let Some(base) = base.get(0) {
                    result.insert(base.to_string());
                }
            }
        }

        if let Err(e) = sender.send(result) {
            error!("ListPath: cannot send result: {:?}", e);
        }
    }
}
