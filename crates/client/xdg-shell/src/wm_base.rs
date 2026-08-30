use std::sync::Arc;

use cosplay_core_client::*;
use cosplay_protocols_xdg_shell::xdg_wm_base::{Ping, Pong, XdgWmBase, XdgWmBaseEvent};
use cosplay_wayland_client::compositor::{CompositorHandle, Empty};

use crate::*;

pub struct XdgWmBaseGlobal {
    handle: WmBaseHandle,
    task: WmHandlePingPong,
}

impl GlobalHandle for XdgWmBaseGlobal {
    type Interface = XdgWmBase;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        let (request_handle, event_handle) = handle.into_split();

        let request_handle = Arc::new(request_handle);

        let handle = WmBaseHandle { request_handle: request_handle.clone() };

        let task = WmHandlePingPong { request_handle, event_handle };

        Self { handle, task }
    }
}

impl XdgWmBaseGlobal {
    pub fn into_parts(self) -> (WmBaseHandle, WmHandlePingPong) {
        let Self { handle, task } = self;
        (handle, task)
    }
}

pub struct WmBaseHandle {
    // FIXME: Graceful destructor request: "Destroying a bound xdg_wm_base object while there are surfaces still alive created by this
    // xdg_wm_base object instance is illegal and will result in a defunct_surfaces error."
    pub(super) request_handle: Arc<RequestHandle<XdgWmBase>>,
}

impl WmBaseHandle {
    pub async fn create_toplevel(&self, compositor: &CompositorHandle) -> Result<ToplevelHandle<Empty>, ToplevelCreateError> {
        ToplevelHandle::new(self, compositor).await
    }
}

pub struct WmHandlePingPong {
    request_handle: Arc<RequestHandle<XdgWmBase>>,
    event_handle: EventHandle<XdgWmBase>,
}

impl WmHandlePingPong {
    pub async fn run(mut self) {
        loop {
            match self.event_handle.recv().await {
                Ok(XdgWmBaseEvent::Ping(Ping { serial })) => {
                    tracing::debug!(serial, "Received ping request. Returning pong.");

                    if self.request_handle.enqueue(Pong { serial }).is_err() {
                        tracing::info!("Request channel closed. Aborting");
                        return;
                    }
                }
                Err(error) => match error {
                    ObjectEventsError::MediatorDown => {
                        tracing::info!("Event channel closed. Aborting.");
                        break;
                    }
                    ObjectEventsError::Convert(error) => {
                        tracing::error!(%error, "Failed to deserialize inbound event.");
                    }
                    ObjectEventsError::ErrorEvent { code, message } => {
                        tracing::warn!(code, message, "Received unhandled error event.");
                    }
                },
            }
        }
    }
}
