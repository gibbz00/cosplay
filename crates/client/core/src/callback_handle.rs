use std::{
    pin::Pin,
    task::{Context, Poll},
};

use cosplay_protocols_wayland::wl_callback::{WlCallback, WlCallbackEvent};

use crate::*;

#[pin_project::pin_project]
pub struct CallbackHandle {
    #[pin]
    handle: ObjectHandle<WlCallback>,
}

impl CallbackHandle {
    // Should only be invoked directly from `RequestHandle::init_callback`.
    // This ensures that no request or events have been interfered with before
    // calling `receive()`.
    pub(crate) fn new(handle: ObjectHandle<WlCallback>) -> Self {
        CallbackHandle { handle }
    }
}

impl Future for CallbackHandle {
    type Output = Result<u32, ObjectEventsError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().handle.project().event.poll_recv(cx).map(|result| {
            result.map(|inbound| match inbound {
                WlCallbackEvent::Done(done) => done.callback_data,
            })
        })
    }
}
