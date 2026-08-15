use std::collections::HashMap;

use cosplay_xml::Cname;

/// Used to map qualified interface names to external packages.
///
/// The generator will default qualify the Rust item as `crate::interface_name::Item`.
/// By registering `interface_name` => `::crate_name` with [`ExternalInterfaces::insert`], the
/// generator will instead qualify the Rust item as `::crate_name::interface_name::Item`.
#[derive(Debug, Default, PartialEq)]
pub struct ExternalInterfaces {
    inner: HashMap<Cname, syn::Path>,
}

impl ExternalInterfaces {
    /// Register an interface to be qualified with the provided path, instead of the local `crate`
    /// identifier.
    ///
    /// External crate paths should be prefixed with `::` in order to avoid naming
    /// collisions with downstream user modules.
    ///
    /// Returns the previously inserted path, if some.
    pub fn insert(&mut self, interface_name: Cname, external_path: syn::Path) -> Option<syn::Path> {
        self.inner.insert(interface_name, external_path)
    }

    pub(crate) fn get(&self, interface_name: &Cname) -> Option<&syn::Path> {
        self.inner.get(interface_name)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    impl<'de> serde::Deserialize<'de> for ExternalInterfaces {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let mut this = Self::default();

            let mappings = HashMap::<String, Vec<Cname>>::deserialize(deserializer)?;

            for (path_str, external_interfaces) in mappings {
                let path = syn::parse_str::<syn::Path>(&path_str).unwrap();

                for interface_name in external_interfaces {
                    if this.insert(interface_name.clone(), path.clone()).is_some() {
                        let message = format!("Duplicate external mapping found. '{interface_name}' has already been registered..",);
                        return Err(serde::de::Error::custom(message));
                    }
                }
            }

            Ok(this)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::*;

        #[test]
        fn from_toml() {
            let toml = r#"
                [external_interfaces]
                foo = ["a", "b"]
                bar = ["c", "d"]
            "#;

            let mut external_interfaces = ExternalInterfaces::default();
            insert(&mut external_interfaces, "a", "foo");
            insert(&mut external_interfaces, "b", "foo");
            insert(&mut external_interfaces, "c", "bar");
            insert(&mut external_interfaces, "d", "bar");

            let expected = GeneratorConfig { external_interfaces, name_mappings: Default::default() };

            let actual = toml::from_str::<GeneratorConfig>(toml).unwrap();

            assert_eq!(actual, expected)
        }

        fn insert(external_interfaces: &mut ExternalInterfaces, interface_name: &str, path: &str) {
            let interface_name = Cname::parse(interface_name.to_string()).unwrap();
            let path = syn::parse_str(path).unwrap();
            external_interfaces.insert(interface_name, path);
        }

        #[test]
        fn duplicate_entry_error() {
            let toml = r#"
                [external_interfaces]
                crate_a = ["interface_a"]
                crate_b = ["interface_a"]
            "#;

            let error = toml::from_str::<GeneratorConfig>(toml).unwrap_err();

            assert!(error.message().contains("Duplicate external mapping found."));
        }
    }
}
