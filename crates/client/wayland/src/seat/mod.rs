mod device_type;
pub(crate) use device_type::{DeviceHandle, Keyboard, Pointer, Touch};

mod capability_broadcast;
pub(crate) use capability_broadcast::{CapabilityBroadcast, CapabilityRemovedRx};

mod handle;
pub use handle::{SeatGetInputError, SeatHandle};

mod pointer;
pub use pointer::PointerHandle;

mod keyboard;
pub use keyboard::KeyboardHandle;

mod touch;
pub use touch::TouchHandle;
