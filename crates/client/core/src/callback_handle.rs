use cosplay_protocols_wayland::wl_callback::{Done, WlCallback, WlCallbackEvent};

use crate::*;

pub struct CallbackHandle {
    handle: ObjectHandle<WlCallback>,
}

impl CallbackHandle {
    pub(crate) fn new(handle: ObjectHandle<WlCallback>) -> Self {
        CallbackHandle { handle }
    }

    pub async fn receive(mut self) -> Result<Done, ObjectEventsError> {
        match self.handle.event().recv().await {
            Some(Ok(ObjectEvent::Event(WlCallbackEvent::Done(done)))) => Ok(done),
            Some(Ok(ObjectEvent::Error { .. })) => unreachable!("Display error not intercepted by mediator."),
            Some(Err(inbound_error)) => Err(ObjectEventsError::Convert(inbound_error)),
            None => Err(ObjectEventsError::MediatorDown),
        }
    }
}
