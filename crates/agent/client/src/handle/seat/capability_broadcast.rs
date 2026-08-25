use std::marker::PhantomData;

const BROADCAST_CAPACITY: usize = 1;

// IMPROVEMENT: device type-parameter?
pub type RemovedCapabilityTx<T> = tokio::sync::broadcast::Sender<PhantomData<T>>;
pub type CapabilityRemovedRx<T> = tokio::sync::broadcast::Receiver<PhantomData<T>>;

/// Sends a close broadcast on drop.
pub struct CapabilityBroadcast<T> {
    tx: RemovedCapabilityTx<T>,
}

impl<T> CapabilityBroadcast<T> {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(BROADCAST_CAPACITY);
        Self { tx }
    }

    pub fn subscribe(&self) -> CapabilityRemovedRx<T> {
        self.tx.subscribe()
    }
}

impl<T> Drop for CapabilityBroadcast<T> {
    fn drop(&mut self) {
        let _ = self.tx.send(PhantomData);
    }
}
