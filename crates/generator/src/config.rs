//! Items for modifying generator output.

use std::collections::HashMap;

use async_wayland_xml::CnameSuffix;

/// Configuration for modifying generator output.
#[derive(Debug, Default)]
#[allow(missing_docs)]
pub struct GeneratorConfig {
    pub name_mappings: NameMappings,
}

/// Provided to [`NameMappings::insert`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Enum,
    // TODO: add Interface, Request, and Event
}

/// Used to translate items from one cname to another.
///
/// Name collision tend to occur given how the generator places all items
/// under the same interface in the same module. `name_mappings` allows one
/// to definee an alternative name for a given [`ItemType`].
///
/// For example; `wl_pointer` has an event named `axis`, but also an enum of
/// the same name. This would then cause a "multiple definitions" compiler
/// error. An `("axis", ItemType::Enum) => "axis_direction"` can then be
/// added in name mapping to avoid creating an enum of the same name.
#[derive(Debug, Default)]
pub struct NameMappings {
    inner: HashMap<(CnameSuffix, ItemType), CnameSuffix>,
}

impl NameMappings {
    /// Insert a new translation entry. See the [`NameMappings`] documentation for more.
    pub fn insert(&mut self, name: CnameSuffix, item: ItemType, new_name: CnameSuffix) {
        self.inner.insert((name, item), new_name);
    }

    pub(crate) fn get(&self, name: &CnameSuffix, item: ItemType) -> Option<&CnameSuffix> {
        // IMPROVEMENT: Remove clone. (&a, &b) -> &(c, d) problem.
        let key = (name.clone(), item);
        self.inner.get(&key)
    }
}
