use super::connection::{Connection, MessageSender};
use dbus_message_parser::message::Message;
use futures::channel::mpsc::{Sender as MpscSender, TrySendError};
use futures::channel::oneshot::Sender as OneshotSender;

impl Connection {
    fn send(&mut self, mut msg: Message) -> Result<u32, TrySendError<Message>> {
        // Increment the serial number.
        self.serial += 1;
        msg.set_serial(self.serial);

        // Send the message.
        self.message_sender.unbounded_send(msg)?;
        Ok(self.serial)
    }

    pub(super) fn send_message(&mut self, msg: Message) {
        if let Err(e) = self.send(msg) {
            error!("could not send msg: {:?}", e);
        }
    }

    pub(super) fn send_message_oneshot(&mut self, msg: Message, response: OneshotSender<Message>) {
        match self.send(msg) {
            Ok(reply_serial) => {
                // Add the response sender to the Map.
                let response = MessageSender::Oneshot(response);
                self.replies.put(reply_serial, response);
            }
            Err(e) => {
                error!("could not send msg: {:?}", e);
            }
        }
    }

    pub(super) fn send_message_mpsc(
        &mut self,
        msg: Message,
        response_reply_serial: OneshotSender<u32>,
        response: MpscSender<Message>,
    ) {
        match self.send(msg) {
            Ok(reply_serial) => {
                if let Err(e) = response_reply_serial.send(reply_serial) {
                    error!("could not send reply serial: {:?}", e);
                }
                // Add the response sender to the Map.
                let response = MessageSender::Mpcs(response);
                self.replies.put(reply_serial, response);
            }
            Err(e) => {
                error!("could not send msg: {:?}", e);
            }
        }
    }
}
