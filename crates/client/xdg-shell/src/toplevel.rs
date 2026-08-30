use std::{collections::HashSet, hash::Hash};

use cosplay_agent::geometry::Dimensions;
use cosplay_codec::{Enumeration, ReleaseRequest};
use cosplay_core_client::{ObjectEventsError, ObjectHandle, RequestError};
use cosplay_protocols_xdg_shell::{
    xdg_surface::{AckConfigure, Configure, GetToplevel, XdgSurface, XdgSurfaceEvent},
    xdg_toplevel::{self, ConfigureBounds, State, WmCapabilities, WmCapability, XdgToplevel, XdgToplevelEvent},
    xdg_wm_base::GetXdgSurface,
};
use cosplay_wayland_client::{
    compositor::{CompositorHandle, Empty, Pending, SurfaceHandle},
    shared_memory::{Available, Committed, ShmBuffer},
};

use crate::*;

pub struct ToplevelHandle<S> {
    wayland_surface_handle: SurfaceHandle<S>,
    inner: Inner,
}

impl<S> AsRef<SurfaceHandle<S>> for ToplevelHandle<S> {
    fn as_ref(&self) -> &SurfaceHandle<S> {
        &self.wayland_surface_handle
    }
}

/// Required to be separated from ToplevelHandle for spread syntax to work
/// together with custom drop order.
struct Inner {
    // NB: handles not wrapped in `ScopedObjectHandle` for manual
    // destructor request ordering in Drop implementation.
    xdg_surface: ObjectHandle<XdgSurface>,
    toplevel: ObjectHandle<XdgToplevel>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // "An xdg_surface must only be destroyed after its role object has been
        // destroyed, otherwise a defunct_role_object error is raised."
        let _ = self.toplevel.request().enqueue(<XdgToplevel as ReleaseRequest>::message());
        let _ = self.xdg_surface.request().enqueue(<XdgSurface as ReleaseRequest>::message());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToplevelCreateError {
    #[error("Failed to synchronize toplevel surface configuration: {0}")]
    Sync(#[from] ToplevelStateError),
    #[error("Failed to encqueue request.")]
    Request(#[from] RequestError),
    #[error("Failed to receive inbound event: {0}")]
    Event(#[from] ObjectEventsError),
    #[error("Server requested the toplevel to be closed.")]
    Close,
}

impl ToplevelHandle<Empty> {
    pub(super) async fn new(base_handle: &WmBaseHandle, compositor_handle: &CompositorHandle) -> Result<Self, ToplevelCreateError> {
        // IMPROVEMENT: log and improve error messaging?

        // Encapsulated surface creation in order to prevent users from passing
        // a surface that has a a role or a buffer already attached.
        //
        // - "A role must be assigned before any other requests are made to the xdg_surface object."
        // - "Creating an xdg_surface from a wl_surface which has a buffer attached or committed is a client
        //   error."
        let wayland_surface = compositor_handle.create_surface()?;

        let xdg_surface = base_handle
            .request_handle
            .init_subobject(|id| GetXdgSurface { id, surface: wayland_surface.id() })?;

        let toplevel = xdg_surface.request().init_subobject(|id| GetToplevel { id })?;

        // TODO: Intermediary toplevel setup before first commit? (Title, app ID etc.)

        // "After creating a role-specific object and setting it up (e.g. by sending the title, app ID, size
        // constraints, parent, etc), the client must perform an initial commit without any buffer
        // attached."
        wayland_surface.commit()?;

        let mut this = Self {
            wayland_surface_handle: wayland_surface,
            inner: Inner { xdg_surface, toplevel },
        };

        // "The client must acknowledge it and is then allowed to attach a buffer to map the surface."
        match this.next_state().await? {
            TopLevelState::ShouldClose => Err(ToplevelCreateError::Close),
            TopLevelState::Configure { serial, .. } => {
                // IMPROVEMENT: Include received configure state in return?
                // Currently dismissed since some servers, i.e. Sway only seems
                // to send any meaningful changes until after the first buffer
                // is attached and committed.

                this.inner.xdg_surface.request().enqueue(AckConfigure { serial: serial.0 })?;

                Ok(this)
            }
        }
    }

    pub fn attach(self, buffer: Option<ShmBuffer<Available>>) -> Result<ToplevelHandle<Pending>, RequestError> {
        let Self { wayland_surface_handle, inner: handles } = self;

        let wayland_surface_handle = wayland_surface_handle.attach(buffer)?;

        Ok(ToplevelHandle { wayland_surface_handle, inner: handles })
    }
}

impl ToplevelHandle<Pending> {
    pub fn commit(self) -> Result<(ToplevelHandle<Empty>, Option<ShmBuffer<Committed>>), RequestError> {
        let Self { wayland_surface_handle, inner: handles } = self;

        let (surface, buffer) = wayland_surface_handle.commit()?;

        let this = ToplevelHandle { wayland_surface_handle: surface, inner: handles };

        Ok((this, buffer))
    }
}

#[derive(Debug)]
pub enum TopLevelState {
    ShouldClose,
    Configure {
        serial: ToplevelConfigSerial,
        change: ToplevelConfigChange,
    },
}

/// From `xdg_surface::configure`.
///
/// Used acknowledge the configuration hint with a corresponding [`ToplevelHandle::ack_configure`].
///
/// Newtype wrapper which does intentionally not implement Clone nor Copy as
/// a means to avoid reuse. "It is an error to issue multiple ack_configure
/// requests referencing a serial from the same configure"
#[derive(Debug)]
pub struct ToplevelConfigSerial(u32);

#[derive(Debug, Default, PartialEq)]
pub struct ToplevelConfigChange {
    /// From `xdg_toplevel::configure_bounds`
    ///
    /// Can for example be used the recommended size passed to `wl_shm_pool`.
    pub bounds_hint: Option<Dimensions>,
    /// From `xdg_toplevel::configure`
    ///
    /// Recommended surface size.
    pub size_hint: Option<Dimensions>,
    /// From `xdg_toplevel::configure`
    pub states: HashSet<State>,
    /// From `xdg_toplevel::wm_capabilities`
    pub capabilities: HashSet<WmCapability>,
}

#[derive(Debug, thiserror::Error)]
pub enum ToplevelStateError {
    #[error("Failed to convert the provided width {0} or height {1} into an u16.")]
    IntegerOverflow(i32, i32),
    #[error("Failed to process object event: {0}")]
    Event(#[from] ObjectEventsError),
}

impl<S> ToplevelHandle<S> {
    /// Will only wait if config context events have been received, but not the final
    /// `xdg_surface::configure`.
    pub async fn try_next_state(&mut self) -> Result<Option<TopLevelState>, ToplevelStateError> {
        self.next_state_impl(false).await
    }

    /// Wait until the next `xdg_toplevel::close` or `xdg_surface::configure` is received.
    pub async fn next_state(&mut self) -> Result<TopLevelState, ToplevelStateError> {
        self.next_state_impl(true)
            .await
            .map(|sync| sync.expect("Impl did not wait for xdg_surface.configure."))
    }

    async fn next_state_impl(&mut self, force_sync: bool) -> Result<Option<TopLevelState>, ToplevelStateError> {
        let mut config_serial = None;
        let mut config_change = ToplevelConfigChange::default();
        let mut received_config_change = false;

        // TODO: Drain wl_surface and check for:
        // - wl_surface.preferred_buffer_scale?
        // - wl_surface.preferred_buffer_transform?

        for inbound_result in self.inner.toplevel.event().iter() {
            if process_inbound(inbound_result?, &mut config_change)? {
                return Ok(Some(TopLevelState::ShouldClose));
            }

            received_config_change = true;
        }

        for inbound_result in self.inner.xdg_surface.event().iter() {
            config_serial = Some(extract_serial(inbound_result?));
        }

        let sync = match config_serial {
            Some(serial) => Some(TopLevelState::Configure { serial, change: config_change }),
            // Check also that config changes were received but no xdg_surface.configure.
            // Can happen if `sync_config()` raced with event forward from server. Wait
            // therefore for it to arrive.
            //
            // Not enough to check if config_change != Default::default() since some
            // config events set the default values.
            None if received_config_change || force_sync => {
                // NB: Biased tokio select required in case more configure context
                // events arrive before the final xdg_surface.configure.
                loop {
                    tokio::select! {
                        biased;

                        xdg_toplevel_event = self.inner.toplevel.event().recv() => {
                            if process_inbound(xdg_toplevel_event?, &mut config_change)? {
                                return Ok(Some(TopLevelState::ShouldClose));
                            }
                        }

                        xdg_surface_event = self.inner.xdg_surface.event().recv() => {
                            let serial = xdg_surface_event.map(extract_serial)?;
                            break Some(TopLevelState::Configure { serial, change: config_change });
                        }
                    }
                }
            }
            None => None,
        };

        return Ok(sync);

        fn process_inbound(event: XdgToplevelEvent, config_change: &mut ToplevelConfigChange) -> Result<bool, ToplevelStateError> {
            match event {
                XdgToplevelEvent::Close(_) => return Ok(true),
                XdgToplevelEvent::Configure(xdg_toplevel::Configure { width, height, states }) => {
                    config_change.size_hint = new_dimensions(width, height)?;

                    // Assuming it's the same as how capabilities should be parsed.
                    //
                    // Not associating enum repr as part of the enum declaration feels
                    // like such an oversight. Sigh.
                    config_change.states = parse_enum_array(states);
                }
                XdgToplevelEvent::ConfigureBounds(ConfigureBounds { width, height }) => {
                    config_change.size_hint = new_dimensions(width, height)?;
                }
                XdgToplevelEvent::WmCapabilities(WmCapabilities { capabilities }) => {
                    config_change.capabilities = parse_enum_array(capabilities);
                }
            }

            Ok(false)
        }

        // For size hint: "If the width or height arguments are zero, it
        // means the client should decide its own window dimension..."
        //
        // For bounds hint: "If width and height are 0, it means bounds
        // is unknown..."
        fn new_dimensions(width: i32, height: i32) -> Result<Option<Dimensions>, ToplevelStateError> {
            if width == 0 && height == 0 {
                return Ok(None);
            }

            Dimensions::new_checked(width, height)
                .ok_or(ToplevelStateError::IntegerOverflow(width, height))
                .map(Some)
        }

        fn extract_serial(event: XdgSurfaceEvent) -> ToplevelConfigSerial {
            match event {
                XdgSurfaceEvent::Configure(Configure { serial }) => ToplevelConfigSerial(serial),
            }
        }
    }
}

fn parse_enum_array<T: Eq + Hash + Enumeration<u32>>(bytes: Vec<u8>) -> HashSet<T> {
    let mut set = HashSet::new();

    let (chunks, _remainder) = bytes.as_chunks();

    for chunk in chunks {
        let repr = u32::from_ne_bytes(*chunk);
        set.insert(T::from_repr(repr));
    }

    set
}

#[cfg(test)]
mod tests {
    use cosplay_core_client::TestDriver;
    use cosplay_protocols_xdg_shell::{xdg_surface, xdg_toplevel};

    use super::*;

    #[test]
    fn drop_toplevel_then_surface() {
        let (mut driver, xdg_surface) = TestDriver::new_raw::<XdgSurface>();
        let toplevel = xdg_surface.request().init_subobject(|id| GetToplevel { id }).unwrap();

        let surface_id = xdg_surface.id();
        let toplevel_id = toplevel.id();

        drop(Inner { xdg_surface, toplevel });

        driver.assert_outbound_request::<GetToplevel>(surface_id);
        driver.assert_outbound_request::<xdg_toplevel::Destroy>(toplevel_id);
        driver.assert_outbound_request::<xdg_surface::Destroy>(surface_id);
    }

    #[test]
    fn parse_states() {
        assert_parse_array(&[State::Maximized, State::Resizing, State::TiledBottom]);
    }

    #[test]
    fn parse_capabilities() {
        assert_parse_array(&[WmCapability::Maximize, WmCapability::WindowMenu, WmCapability::Fullscreen]);
    }

    fn assert_parse_array<T: std::fmt::Debug + Clone + Copy + Eq + Hash + Enumeration<u32>>(items: &[T]) {
        let mut buffer = Vec::new();

        for item in items {
            let bytes = Enumeration::<u32>::to_repr(item).to_ne_bytes();
            buffer.extend_from_slice(&bytes);
        }

        let actual = parse_enum_array(buffer);

        let expected = HashSet::from_iter(items.iter().copied());

        assert_eq!(expected, actual);
    }
}
