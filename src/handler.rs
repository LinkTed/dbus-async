use crate::{DBus, DBusResult};
use async_trait::async_trait;
use dbus_message_parser::{message::Message, value::ObjectPath};
use futures::{
    channel::mpsc::{channel, Receiver},
    lock::Mutex,
    StreamExt,
};
use std::sync::Arc;

/// A trait for the generic `Message` handler.
#[async_trait]
pub trait Handler: Send {
    /// Handle the `Message`.
    async fn handle(&mut self, dbus: &DBus, msg: Message) -> DBusResult<()>;
}

#[async_trait]
pub trait Binder: Sized {
    async fn bind(self, dbus: DBus, object_path: ObjectPath) -> DBusResult<()> {
        let (sender, receiver) = channel(128);
        dbus.add_method_call(object_path, sender)?;
        self.bind_by_receiver(dbus, receiver).await
    }

    async fn bind_by_receiver(self, dbus: DBus, receiver: Receiver<Message>) -> DBusResult<()>;
}

// TODO: Wait until specialization https://github.com/rust-lang/rust/issues/31844 is finished to
// define a default impl with Deletable
#[async_trait]
impl<T> Binder for T
where
    T: Handler,
{
    async fn bind_by_receiver(
        mut self,
        dbus: DBus,
        mut receiver: Receiver<Message>,
    ) -> DBusResult<()> {
        while let Some(msg) = receiver.next().await {
            self.handle(&dbus, msg).await?;
        }
        Ok(())
    }
}

// TODO: Wait until specialization https://github.com/rust-lang/rust/issues/31844 is finished to
// define a default impl with Deletable
#[async_trait]
impl<T> Binder for Arc<Mutex<T>>
where
    T: Handler,
{
    async fn bind_by_receiver(self, dbus: DBus, mut receiver: Receiver<Message>) -> DBusResult<()> {
        while let Some(msg) = receiver.next().await {
            let mut guard = self.lock().await;
            guard.handle(&dbus, msg).await?;
        }
        Ok(())
    }
}
