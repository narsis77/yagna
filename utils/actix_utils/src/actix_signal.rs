use actix::prelude::*;
use anyhow::{anyhow, Result};

/// Subscribe to process signals.
#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Subscribe<MessageType>(pub Recipient<MessageType>)
where
    MessageType: Message + std::marker::Send + std::marker::Sync + std::clone::Clone,
    MessageType::Result: std::marker::Send + std::marker::Sync;

/// Actor that provides signal subscriptions
pub struct SignalSlot<MessageType>
where
    MessageType: Message + std::marker::Send + std::marker::Sync + std::clone::Clone,
    MessageType::Result: std::marker::Send + std::marker::Sync,
{
    subscribers: Vec<Recipient<MessageType>>,
}

#[allow(dead_code)]
impl<MessageType> SignalSlot<MessageType>
where
    MessageType: Message + std::marker::Send + std::marker::Sync + std::clone::Clone,
    MessageType::Result: std::marker::Send + std::marker::Sync,
{
    pub fn new() -> SignalSlot<MessageType> {
        SignalSlot::<MessageType> {
            subscribers: vec![],
        }
    }

    /// Send signal to all subscribers
    pub fn send_signal(&self, message: MessageType) -> Result<()> {
        let errors = self
            .subscribers
            .iter()
            .map(|subscriber| subscriber.do_send(message.clone()))
            .filter_map(|result| {
                match result {
                    Err(error) => {
                        //TODO: It would be useful to have better error message, that suggest which signal failed.
                        log::error!(
                            "Sending signal to subscriber failed in SignalSlot::send_signal. {}",
                            error
                        );
                        Some(error)
                    }
                    Ok(_) => None,
                }
            })
            .collect::<Vec<SendError<MessageType>>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!("Errors while sending signal: {:?}", errors))
        }
    }

    pub fn subscribe(&mut self, subscriber: Recipient<MessageType>) {
        self.subscribers.push(subscriber);
    }

    pub fn on_subscribe(&mut self, msg: Subscribe<MessageType>) {
        self.subscribe(msg.0);
    }
}
