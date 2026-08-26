use cosplay_codec::{Interface, OpaqueNewObjectId, OpaqueObjectId};
use cosplay_protocols_wayland::wl_registry::{self, WlRegistry, WlRegistryEvent};

use crate::*;

/// Intended to only be implemented for handles retrievable from [`RegistryHandle::bind`].
pub trait GlobalHandle {
    type Interface;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self;
}

pub struct RegistryHandle {
    object_handle: ObjectHandle<WlRegistry>,
    registry_map: RegistryMap,
}

/// Returned from [`RegistryHandle::bind_raw`] and [`RegistryHandle::bind`].
#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error("Global not present in registry.")]
    NotRegistered,
    #[error("General request error: {0}")]
    Common(#[from] RequestError),
    #[error("Failed to retrieve object events: {0}")]
    Events(#[from] ObjectEventsError),
}

impl RegistryHandle {
    pub(crate) fn new(object_handle: ObjectHandle<WlRegistry>) -> Self {
        Self { object_handle, registry_map: Default::default() }
    }

    /// Bind to a global handle wrapper `H`
    ///
    /// These handles encapsulate common interface-specific logic. Users that
    /// want some more granular control over the bound object can instead use
    /// [`Self::bind_raw`].
    pub async fn bind<H: GlobalHandle>(&mut self) -> Result<H, BindError>
    where
        H::Interface: Interface,
    {
        self.bind_raw::<H::Interface>().await.map(H::from_raw)
    }

    /// Create a raw [`ObjectHandle`] for a registered global instance of `I`.
    ///
    /// Many object will have wrappers, just like RegistryHandle is a wrapper
    /// for `ObjectHandle<WlRegistry>`, for encapsulating interface-specific
    /// logic. Such globals can instead be bound by using [`Self::bind`].
    pub async fn bind_raw<I: Interface>(&mut self) -> Result<ObjectHandle<I>, BindError> {
        // Make sure that map is up to date.
        for inbound_result in self.object_handle.events_iter() {
            match inbound_result? {
                ObjectEvent::Event(event) => match event {
                    WlRegistryEvent::Global(global) => {
                        self.registry_map.register(global);
                    }
                    WlRegistryEvent::GlobalRemove(global_remove) => {
                        self.registry_map.remove(global_remove);
                    }
                },
                ObjectEvent::Error { code, message } => {
                    // Assuming that this should not occur so long as
                    // the registry implementation is correct? Hard to
                    // tell from the wayland.xml
                    tracing::error!(code, message, "Registry received an unhandled error.")
                }
            }
        }

        let RegistryEntry { number_name, server_version } = self.registry_map.get::<I>().ok_or(BindError::NotRegistered)?;

        let resolved_version = std::cmp::min(*server_version, I::VERSION);

        let interface_name = I::NAME.to_string();

        let subobject = self
            .object_handle
            .init_subobject_with_version(resolved_version, |new_id| wl_registry::Bind {
                name: *number_name,
                id: OpaqueNewObjectId {
                    // Seems a bit superfluous given the name argument. Regardless,
                    // most compositors will error if this does not match with the
                    // name sent over `wl_registry::global`.
                    interface_name,
                    interface_version: resolved_version,
                    object_id: OpaqueObjectId::new(new_id.inner()),
                },
            })?;

        Ok(subobject)
    }
}

// TODO:
// impl Drop {
//     fn drop() {
//         // If wl_fixes exists in registry map, send wl_fixes.destroy_registry()
//     }
// }
//

mod map {
    use std::collections::HashMap;

    use cosplay_codec::Interface;
    use cosplay_protocols_wayland::wl_registry::{Global, GlobalRemove};

    #[derive(Default)]
    pub(crate) struct RegistryMap {
        inner: HashMap<String, RegistryEntry>,
    }

    #[derive(Debug, PartialEq)]
    pub(crate) struct RegistryEntry {
        pub(crate) number_name: u32,
        pub(crate) server_version: u32,
    }

    impl RegistryMap {
        pub fn register(&mut self, global: Global) {
            let Global { name, interface, version } = global;

            tracing::debug!(
                number_name = name,
                inferace_name = interface,
                server_version = version,
                "Registering new global."
            );

            let entry = RegistryEntry { number_name: name, server_version: version };

            self.inner.insert(interface, entry);
        }

        pub fn remove(&mut self, global_remove: GlobalRemove) -> Option<RegistryEntry> {
            tracing::debug!(number_name = global_remove.name, "Removing global.");

            // Probably not the end of the world performance-wise, given the simplicity traded for.
            self.inner
                .iter()
                .find_map(|(name, entry)| (entry.number_name == global_remove.name).then_some(name))
                .cloned()
                .and_then(|name| self.inner.remove(&name))
        }

        pub fn get<I: Interface>(&self) -> Option<&RegistryEntry> {
            self.inner.get(I::NAME)
        }
    }

    #[cfg(test)]
    mod tests {
        use cosplay_protocols_wayland::wl_shm::WlShm;

        use super::*;

        fn mock_entry() -> RegistryEntry {
            RegistryEntry { number_name: 123, server_version: 456 }
        }

        fn mock_global() -> Global {
            Global { name: 123, interface: WlShm::NAME.to_string(), version: 456 }
        }

        fn mock_global_remove() -> GlobalRemove {
            GlobalRemove { name: 123 }
        }

        #[test]
        fn register() {
            let mut map = RegistryMap::default();

            assert!(map.get::<WlShm>().is_none());

            map.register(mock_global());

            let stored = map.get::<WlShm>().unwrap();

            let expected = mock_entry();

            assert_eq!(&expected, stored);
        }

        #[test]
        fn remove() {
            let mut map = RegistryMap::default();

            assert!(map.remove(mock_global_remove()).is_none());

            map.register(mock_global());

            assert!(!map.inner.is_empty());

            let removed = map.remove(mock_global_remove()).unwrap();

            assert!(map.inner.is_empty());

            let expected = mock_entry();

            assert_eq!(expected, removed);
        }
    }
}
pub(super) use map::{RegistryEntry, RegistryMap};
