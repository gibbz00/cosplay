/// Interface declaration trait.
pub trait Interface {
    /// Used to create or match the interface name sent in `wl_registry::global`.
    const NAME: &str;

    /// Defines the version supported supported by the framework.
    const VERSION: u32;
}
