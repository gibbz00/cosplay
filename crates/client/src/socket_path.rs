//! `libwayland` like socket path resolution.
//!
//! See [`SocketPath::resolve`] for more.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

/// Returned from [`SocketPath::resolve`].
#[allow(missing_docs)]
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SocketPathError<'a> {
    #[error("'$XDG_RUNTIME_DIR' not set. Required to the relative socket path '{0}' absolute.")]
    XdgRuntimeNotFound(Cow<'a, Path>),
}

/// Path wrapper created with [`Self::resolve`].
///
/// Implements `AsRef<Path>` for retrieving the inner path.

#[derive(Debug)]
pub struct SocketPath<'a>(Cow<'a, Path>);

impl AsRef<Path> for SocketPath<'_> {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl<'a> SocketPath<'a> {
    const ENV_KEY: &'static str = "WAYLAND_DISPLAY";
    const FALLBACK_PATH: &'static str = "./wayland-0";
    const XDG_KEY: &'static str = "XDG_RUNTIME_DIR";

    /// Resolve the unix socket path created by the wayland server.
    ///
    /// Resolution is similar to `libwayland`'s [wl_display_connect] and
    /// done using the following strategy:
    ///
    /// A path selected by first checking if one is given in `provided_path`. If
    /// none, the resolver checks if `$WAYLAND_DISPLAY` is set, before falling back
    /// to `./wayland-0`.
    ///
    /// Next, the resolver checks if the found path is relative or absolute.
    /// Absolute paths are returned immediatedly, relative ones are prepended with
    /// `$XDG_RUNTIME_DIR`.
    ///
    /// [wl_display_connect]: https://wayland.freedesktop.org/docs/html/apb.html#Client-classwl__display_1af048371dfef7577bd39a3c04b78d0374
    pub fn resolve(provided_path: Option<&'a Path>) -> Result<Self, SocketPathError<'a>> {
        let path = provided_path
            .map(Cow::Borrowed)
            .or_else(|| std::env::var_os(Self::ENV_KEY).map(PathBuf::from).map(Cow::Owned))
            .unwrap_or(Path::new(Self::FALLBACK_PATH).into());

        if path.is_absolute() {
            return Ok(Self(path));
        }

        let Some(xdg_path) = std::env::var_os(Self::XDG_KEY).map(PathBuf::from) else {
            return Err(SocketPathError::XdgRuntimeNotFound(path));
        };

        Ok(Self(xdg_path.join(path).into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn resolve_provided_absolute() {
        prepare_env(None, None);

        let provided = Path::new("/absolute/provided");
        let actual = SocketPath::resolve(Some(provided)).unwrap();

        assert_eq!(provided, actual.as_ref());
    }

    #[test]
    #[serial_test::serial]
    fn resolve_provided_relative() {
        prepare_env(Some("/tmp"), None);

        let actual = SocketPath::resolve(Some(Path::new("./provided"))).unwrap();
        let expected = Path::new("/tmp/./provided");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial]
    fn resolve_env_absolute() {
        prepare_env(None, Some("/absolute/test"));

        let actual = SocketPath::resolve(None).unwrap();
        let expected = Path::new("/absolute/test");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial]
    fn resolve_env_relative() {
        prepare_env(Some("/tmp"), Some("./test"));

        let actual = SocketPath::resolve(None).unwrap();
        let expected = Path::new("/tmp/./test");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial]
    fn resolve_fallback() {
        let prefix = "/tmp";

        prepare_env(Some(prefix), None);

        let actual = SocketPath::resolve(None).unwrap();
        let expected = PathBuf::from(prefix).join(SocketPath::FALLBACK_PATH);

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial]
    fn resolve_no_xdg_runtime_error() {
        prepare_env(None, None);

        let actual_error = SocketPath::resolve(None).unwrap_err();
        let expected_error = SocketPathError::XdgRuntimeNotFound(Path::new(SocketPath::FALLBACK_PATH).into());
        assert_eq!(expected_error, actual_error);
    }

    fn prepare_env(xdg: Option<&str>, wayland_display: Option<&str>) {
        set_or_remove(SocketPath::XDG_KEY, xdg);
        set_or_remove(SocketPath::ENV_KEY, wayland_display);

        fn set_or_remove(name: &str, env: Option<&str>) {
            // SAFETY: tests are run in serial
            match env {
                Some(path) => unsafe { std::env::set_var(name, path) },
                None => unsafe { std::env::remove_var(name) },
            }
        }
    }
}
