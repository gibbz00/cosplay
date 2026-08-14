//! Items for modifying generator output.

use std::collections::HashMap;

use cosplay_xml::{Cname, CnameSuffix};

/// Configuration for modifying generator output.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[allow(missing_docs)]
pub struct GeneratorConfig {
    pub name_mappings: NameMappings,
}

/// Provided to [`NameMappings::insert`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
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
#[derive(Debug, Default, PartialEq)]
pub struct NameMappings {
    inner: HashMap<NameMappingKey, CnameSuffix>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct NameMappingKey {
    interface_name: Cname,
    current_name: CnameSuffix,
    item_type: ItemType,
}

impl NameMappings {
    /// Insert a new translation entry. See the [`NameMappings`] documentation for more.
    pub fn insert(&mut self, interface_name: Cname, current_name: CnameSuffix, item_type: ItemType, new_name: CnameSuffix) {
        let key = NameMappingKey { interface_name, current_name, item_type };
        self.inner.insert(key, new_name);
    }

    pub(crate) fn get(&self, interface_name: &Cname, name: &CnameSuffix, item_type: ItemType) -> Option<&CnameSuffix> {
        // IMPROVEMENT: Remove clones. (&a, &b) -> &(c, d) problem.
        let key = NameMappingKey {
            interface_name: interface_name.clone(),
            current_name: name.clone(),
            item_type,
        };

        self.inner.get(&key)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    impl<'de> serde::Deserialize<'de> for NameMappings {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            struct Mapping {
                enums: HashMap<CnameSuffix, CnameSuffix>,
            }

            let table = HashMap::<Cname, Mapping>::deserialize(deserializer)?;

            let mut this = Self::default();

            for (interface_name, mappings) in table {
                for (current_name, new_name) in mappings.enums {
                    this.insert(interface_name.clone(), current_name, ItemType::Enum, new_name);
                }
            }

            Ok(this)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_toml() {
            let toml = r#"
                [name_mappings.wl_a.enums]
                foo = "foo_bar"

                [name_mappings.wl_b.enums]
                baz = "baz_qux"
            "#;

            let mut name_mappings = NameMappings::default();
            name_mappings.insert(
                Cname::parse("wl_a".to_string()).unwrap(),
                CnameSuffix::parse("foo".to_string()).unwrap(),
                ItemType::Enum,
                CnameSuffix::parse("foo_bar".to_string()).unwrap(),
            );
            name_mappings.insert(
                Cname::parse("wl_b".to_string()).unwrap(),
                CnameSuffix::parse("baz".to_string()).unwrap(),
                ItemType::Enum,
                CnameSuffix::parse("baz_qux".to_string()).unwrap(),
            );

            let expected = GeneratorConfig { name_mappings };

            let actual = toml::from_str::<GeneratorConfig>(toml).unwrap();

            assert_eq!(actual, expected)
        }
    }
}
