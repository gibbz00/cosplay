// TEMP:
#![allow(missing_docs)]

pub mod shared_memory;
pub(crate) use shared_memory::*;

pub mod compositor;
pub(crate) use compositor::*;

pub mod seat;
pub(crate) use seat::*;
