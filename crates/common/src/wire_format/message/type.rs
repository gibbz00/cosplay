#[sealed::sealed]
pub trait MessageType {}

pub struct Request;

#[sealed::sealed]
impl MessageType for Request {}

pub struct Event;

#[sealed::sealed]
impl MessageType for Event {}
