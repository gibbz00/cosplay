use cosplay_codec::{DecodeMessage, Interface, Message, OpaqueMessage};

pub struct MessageUtils;

impl MessageUtils {
    /// Attempt to deserialize an opaque message, returned Some if successful.
    ///
    /// Deserialization errors are logged before returned None.
    ///
    /// Used when there isn't much a library user can do if deserialization errors do occur.
    /// I.e. in an internal event loop.
    pub(crate) fn into_concrete_logged<M: Message + DecodeMessage>(event: OpaqueMessage) -> Option<M>
    where
        M::Interface: Interface,
    {
        let object_id = event.object_id();
        let opcode = event.opcode();

        event
            .into_concrete::<M>()
            .inspect_err(|error| {
                tracing::error!(
                    %object_id,
                    %opcode,
                    interface = <M::Interface as Interface>::NAME,
                    event = M::NAME,
                    %error,
                    "Failed to event into its concrete counterpart."
                );
            })
            .ok()
    }
}
