use std::marker::PhantomData;

use cosplay_codec::{Event, Inbound, IntoInboundError};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

#[impl_tools::autoimpl(Debug)]
pub struct EventHandle<I> {
    pub(crate) inbound_rx: ObjectHandleRx,
    pub(crate) interface_marker: PhantomData<I>,
}

pub enum ObjectEvent<E> {
    Event(E),
    Error { code: u32, message: String },
}

impl<I> EventHandle<I> {
    pub fn iter(&mut self) -> ObjectEventsIter<'_, I> {
        ObjectEventsIter::new(&mut self.inbound_rx)
    }

    pub async fn recv(&mut self) -> Option<Result<ObjectEvent<I::Enum>, IntoInboundError>>
    where
        I: Inbound<Event>,
    {
        self.inbound_rx.recv().await.map(|event| match event {
            ObjectHandleMessage::Event(message) => I::from_opaque(message).map(ObjectEvent::Event),
            ObjectHandleMessage::Error { code, message } => Ok(ObjectEvent::Error { code, message }),
        })
    }
}

pub struct ObjectEventsIter<'a, I> {
    inbound_rx: &'a mut ObjectHandleRx,
    interface_marker: PhantomData<I>,
}

impl<'a, I> ObjectEventsIter<'a, I> {
    pub(super) fn new(inbound_rx: &'a mut ObjectHandleRx) -> Self {
        Self { inbound_rx, interface_marker: PhantomData }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectEventsError {
    #[error("Mediator down. Unable to receive any new events.")]
    MediatorDown,
    #[error("Failed to convert opaque message into inbound event: {0}")]
    Convert(#[from] IntoInboundError),
}

impl<I> Iterator for ObjectEventsIter<'_, I>
where
    I: Inbound<Event>,
{
    type Item = Result<ObjectEvent<I::Enum>, ObjectEventsError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inbound_rx.try_recv() {
            Ok(event) => Some(match event {
                ObjectHandleMessage::Event(message) => I::from_opaque(message).map(ObjectEvent::Event).map_err(Into::into),
                ObjectHandleMessage::Error { code, message } => Ok(ObjectEvent::Error { code, message }),
            }),
            Err(error) => match error {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => Some(Err(ObjectEventsError::MediatorDown)),
            },
        }
    }
}
