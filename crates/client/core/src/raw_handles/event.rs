use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use cosplay_codec::{Event, Inbound, IntoInboundError};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

#[impl_tools::autoimpl(Debug)]
pub struct EventHandle<I> {
    pub(crate) inbound_rx: ObjectHandleRx,
    pub(crate) interface_marker: PhantomData<I>,
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectEventsError {
    #[error("Mediator down. Unable to receive any new events.")]
    MediatorDown,
    #[error("Failed to convert opaque message into inbound event: {0}")]
    Convert(#[from] IntoInboundError),
    #[error("Received error event forwarded through wl_display. Code '{code}; message: {message}")]
    ErrorEvent { code: u32, message: String },
}

impl<I> EventHandle<I> {
    pub fn iter(&mut self) -> ObjectEventsIter<'_, I> {
        ObjectEventsIter::new(&mut self.inbound_rx)
    }

    // IMPROVEMENT: Implement stream?

    pub async fn recv(&mut self) -> Result<I::Enum, ObjectEventsError>
    where
        I: Inbound<Event>,
    {
        let message = self.inbound_rx.recv().await.ok_or(ObjectEventsError::MediatorDown)?;
        convert_message::<I>(message)
    }

    pub fn poll_recv(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<I::Enum, ObjectEventsError>>
    where
        I: Inbound<Event> + Unpin,
    {
        self.inbound_rx.poll_recv(cx).map(|message| {
            let message = message.ok_or(ObjectEventsError::MediatorDown)?;
            convert_message::<I>(message)
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

impl<I> Iterator for ObjectEventsIter<'_, I>
where
    I: Inbound<Event>,
{
    type Item = Result<I::Enum, ObjectEventsError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inbound_rx.try_recv() {
            Ok(message) => Some(convert_message::<I>(message)),
            Err(error) => match error {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => Some(Err(ObjectEventsError::MediatorDown)),
            },
        }
    }
}

fn convert_message<I>(message: ObjectHandleMessage) -> Result<I::Enum, ObjectEventsError>
where
    I: Inbound<Event>,
{
    match message {
        ObjectHandleMessage::Event(message) => I::from_opaque(message).map_err(Into::into),
        ObjectHandleMessage::Error { code, message } => Err(ObjectEventsError::ErrorEvent { code, message }),
    }
}
