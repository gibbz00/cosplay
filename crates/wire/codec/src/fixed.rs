/// Signed 24.8 decimal type.
///
/// Conversions from i32 and f64 mimic those done by [libwayland](libwayland_impl).
///
/// [libwayland_impl]: https://gitlab.freedesktop.org/wayland/wayland/-/blob/99638501a1314e68c79176fa2cafa3bbe6cf55ea/src/wayland-util.h#L621-673
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fixed {
    pub(crate) integer: i24::I24,
    pub(crate) decimal: u8,
}
