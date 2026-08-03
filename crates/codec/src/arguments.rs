use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::BytesMut;

pub struct ArgumentBody<'a> {
    body: &'a mut BytesMut,
    fd_buffer: &'a mut VecDeque<OwnedFd>,
}
