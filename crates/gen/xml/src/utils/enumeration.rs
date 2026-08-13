use std::collections::HashMap;

use crate::*;

/// Used to map enumeration paths to their u32/i32 reprs declared in the request/event args.
pub struct EnumReprMap {
    inner: HashMap<EnumPath, EnumRepr>,
}

/// Error returned from from [`EnumReprMap::new`].
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum EnumReprMapBuildError {
    #[error("New enum registration conflicts with previously registered repr for '{0}'.")]
    ConflictingRepr(CnameSuffix),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum EnumRepr {
    U32,
    I32,
}

impl EnumReprMap {
    /// Traverse a protocol and store all enum type mentioned in interface requests and events.
    ///
    /// Most code generators need the enum type representation when generating the code for the
    /// enum, and not the argument itself.
    ///
    /// Found mapping can then be retrieved with [`EnumReprMap::get`].
    pub fn new(protocol: &Protocol) -> Result<Self, EnumReprMapBuildError> {
        let mut this = Self { inner: Default::default() };

        for interface in &protocol.interfaces {
            let name = &interface.name;

            for request in &interface.requests {
                this.traverse_message(name, request)?;
            }

            for event in &interface.events {
                this.traverse_message(name, event)?;
            }
        }

        Ok(this)
    }

    /// Retrieve the enum representation for a given interface and enum identifier.
    ///
    /// Returns `None` if the enum is not part of the protocol used to build the map.
    pub fn get(&self, interface: &Cname, enum_ident: &CnameSuffix) -> Option<EnumRepr> {
        // IMPROVEMENT: remove clones, classic (&A, &B) -> &(A, B) issue.
        self.inner
            .get(&EnumPath { interface: Some(interface.clone()), enumeration: enum_ident.clone() })
            .copied()
    }

    fn traverse_message(&mut self, parent_interface: &Cname, message: &Message) -> Result<(), EnumReprMapBuildError> {
        for argument in &message.arguments {
            let enum_meta = match &argument.variant {
                ArgumentVariant::I32 { enumeration: Some(enum_ident) } => Some((enum_ident, EnumRepr::I32)),
                ArgumentVariant::U32 { enumeration: Some(enum_ident) } => Some((enum_ident, EnumRepr::U32)),
                _ => None,
            };

            if let Some((path, repr)) = enum_meta {
                let EnumPath { interface, enumeration } = path;

                let interface = interface.as_ref().unwrap_or(parent_interface);

                self.register(interface.clone(), enumeration.clone(), repr)?;
            }
        }

        Ok(())
    }

    fn register(&mut self, interface: Cname, enum_ident: CnameSuffix, repr: EnumRepr) -> Result<(), EnumReprMapBuildError> {
        // NB: Avoids exposing parameter as EnumPath to force the interface parameter as Some.
        let path = EnumPath { interface: Some(interface), enumeration: enum_ident.clone() };

        if let Some(previous_value) = self.inner.insert(path, repr)
            && previous_value != repr
        {
            return Err(EnumReprMapBuildError::ConflictingRepr(enum_ident));
        };

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn conflicting_repr_err() {
        let interface = Cname("foo".to_string());
        let ident = CnameSuffix("bar".to_string());

        let mut map = EnumReprMap { inner: Default::default() };

        assert!(map.register(interface.clone(), ident.clone(), EnumRepr::U32).is_ok());

        assert!(map.register(interface.clone(), ident.clone(), EnumRepr::U32).is_ok());

        let error = map.register(interface, ident, EnumRepr::I32).unwrap_err();

        assert_matches!(error, EnumReprMapBuildError::ConflictingRepr(_));
    }

    #[test]
    fn skips_non_enum_repr() {
        let protocol = mock_protocol(
            [ArgumentVariant::U32 { enumeration: None }],
            [ArgumentVariant::I32 { enumeration: None }],
        );

        let map = EnumReprMap::new(&protocol).unwrap();

        assert!(map.inner.is_empty());
    }

    #[test]
    fn register_local() {
        let enum_ident = CnameSuffix("xxx".to_string());

        let protocol = mock_protocol(
            [ArgumentVariant::U32 {
                enumeration: Some(EnumPath { interface: None, enumeration: enum_ident.clone() }),
            }],
            None,
        );

        let map = EnumReprMap::new(&protocol).unwrap();

        let repr = map.get(&mock_local_interface(), &enum_ident).unwrap();

        assert_eq!(EnumRepr::U32, repr);
    }

    #[test]
    fn register_external() {
        let enum_ident = CnameSuffix("xxx".to_string());
        let interface_ident = Cname("yyy".to_string());

        let protocol = mock_protocol(
            None,
            [ArgumentVariant::I32 {
                enumeration: Some(EnumPath {
                    interface: Some(interface_ident.clone()),
                    enumeration: enum_ident.clone(),
                }),
            }],
        );

        let map = EnumReprMap::new(&protocol).unwrap();

        assert!(map.get(&mock_local_interface(), &enum_ident).is_none());

        let repr = map.get(&interface_ident, &enum_ident).unwrap();

        assert_eq!(EnumRepr::I32, repr);
    }

    fn mock_local_interface() -> Cname {
        Cname("SomeInterface".to_string())
    }

    fn mock_protocol(
        request_arguments: impl IntoIterator<Item = ArgumentVariant>,
        event_arguments: impl IntoIterator<Item = ArgumentVariant>,
    ) -> Protocol {
        Protocol {
            name: Cname("SomeProtocol".to_string()),
            copyright: None,
            description: None,
            interfaces: vec![Interface {
                name: mock_local_interface(),
                version: Default::default(),
                frozen: Default::default(),
                description: Default::default(),
                requests: vec![Message {
                    name: Cname("SomeRequest".to_string()),
                    destructor: Default::default(),
                    since: Default::default(),
                    deprecated_since: Default::default(),
                    description: Default::default(),
                    arguments: mock_arguments(request_arguments),
                }],
                events: vec![Message {
                    name: Cname("SomeEvent".to_string()),
                    destructor: Default::default(),
                    since: Default::default(),
                    deprecated_since: Default::default(),
                    description: Default::default(),
                    arguments: mock_arguments(event_arguments),
                }],
                enums: Default::default(),
            }],
        }
    }

    fn mock_arguments(variants: impl IntoIterator<Item = ArgumentVariant>) -> Vec<Argument> {
        variants
            .into_iter()
            .map(|variant| Argument {
                name: Cname("SomeArgument".to_string()),
                description: Description { summary: None, text: None },
                variant,
            })
            .collect()
    }
}
