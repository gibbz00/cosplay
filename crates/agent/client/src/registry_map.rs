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

        let entry = RegistryEntry { number_name: name, server_version: version };

        self.inner.insert(interface, entry);
    }

    pub fn remove(&mut self, global_remove: GlobalRemove) -> Option<RegistryEntry> {
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
