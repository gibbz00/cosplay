// TEMP:
#![allow(missing_docs)]

mod xdg_wm_base_handle;
pub use xdg_wm_base_handle::{WmBaseHandle, WmHandlePingPong, XdgWmBaseGlobal};

mod xdg_toplevel;
pub use xdg_toplevel::{ToplevelError, ToplevelHandle};
