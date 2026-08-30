mod region;
pub use region::{CreateShmRegionError, ShmRegion};

mod shm_handle;
pub use shm_handle::{CreateBufferError, WlShmHandle};

mod buffer_handle;
pub use buffer_handle::{Available, Committed, ShmBuffer};
