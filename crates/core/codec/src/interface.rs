/// Interface declaration trait.
pub trait Interface {
    /// Used to create or match the interface name sent in `wl_registry::global`.
    const NAME: &str;

    /// Defines the version supported supported by the framework.
    const VERSION: u32;
}

/// Implemented on types which implement [`Interface`], but which also an
/// explicit destructor **request**.
pub trait ReleaseRequest {
    /// The message of type destructort
    type Message;

    /// Create a new [`Self::Message`].
    fn message() -> Self::Message;
}
