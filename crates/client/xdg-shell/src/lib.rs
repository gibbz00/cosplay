// TEMP:
#![allow(missing_docs)]

mod wm_base_handle;
pub use wm_base_handle::{WmBaseHandle, WmHandlePingPong, XdgWmBaseGlobal};

mod toplevel;
pub use toplevel::{ToplevelError, ToplevelHandle};
