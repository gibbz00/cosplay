use std::ops::RangeInclusive;

use crate::*;

pub(crate) trait ObjectIdBounds: Entity {
    const RANGE: RangeInclusive<u32>;
}

impl ObjectIdBounds for Client {
    const RANGE: RangeInclusive<u32> = 1..=0xFEFFFFFF;
}

impl ObjectIdBounds for Server {
    const RANGE: RangeInclusive<u32> = 0xFF000000..=0xFFFFFFFF;
}
