use async_trait::async_trait;
use dbus_async::{Binder, DBus, DBusResult, Handler};
use dbus_message_parser::message::Message;
use dbus_message_parser::value::Value;
use std::convert::TryInto;

// This is a low level example, where the user defines the Handler trait by himself.

struct UserDefinedObject {}

impl UserDefinedObject {
    fn new() -> UserDefinedObject {
        UserDefinedObject {}
    }
}

#[async_trait]
impl Handler for UserDefinedObject {
    async fn handle(&mut self, dbus: &DBus, msg: Message) -> DBusResult<()> {
        println!("Got message {:?}", msg);
        if let Ok(mut msg) = msg.method_return() {
            msg.add_value(Value::String("Hello world".to_string()));
            println!("Response: Hello world");
            dbus.send(msg)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (dbus, _connection_handle) = DBus::session(true, true)
        .await
        .expect("failed to get the DBus object");
    // Create a object, which implement the `Handle`
    let dbus_object = UserDefinedObject::new();
    // The object path
    let object_path = "/object/path".try_into().unwrap();
    // Bind the object to the dedicated object path
    dbus_object
        .bind(dbus, object_path)
        .await
        .expect("No more message to receive");
}
