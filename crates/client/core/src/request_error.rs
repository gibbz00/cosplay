/// Common request errors.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Failed to allocate a new object ID. No ID available.")]
    NoIdAvailable,
    #[error("Failed to communicate with event mediator.")]
    MediatorDown,
    #[error("Request channel dropped.")]
    RequestQueueDown,
}
