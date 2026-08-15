use std::collections::HashMap;

use cosplay_codec::Interface;
use cosplay_protocols_wayland::wl_registry::{Global, GlobalRemove};

#[derive(Default)]
pub(crate) struct RegistryMap {
    inner: HashMap<String, RegistryEntry>,
}

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

    pub fn remove(&mut self, global_remove: GlobalRemove) {
        // TODO(log): warn if global not found?

        // Probably not the end of the world performance-wise, given the simplicity traded for.
        if let Some(interface_name) = self
            .inner
            .iter()
            .find_map(|(name, entry)| (entry.number_name == global_remove.name).then_some(name))
            .cloned()
        {
            self.inner.remove(&interface_name);
        }
    }

    pub fn get<I: Interface>(&self) -> Option<&RegistryEntry> {
        self.inner.get(I::NAME)
    }
}
