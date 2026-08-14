//! `libwayland`-like socket path resolution. See [`SocketPath::resolve`] for more.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

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

/// Returned from [`SocketPath::resolve`].
#[allow(missing_docs)]
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SocketPathError {
    #[error("'$XDG_RUNTIME_DIR' not set. Required when the provided query path is relative.")]
    XdgRuntimeNotFound,
}

impl<'a> SocketPath<'a> {
    const ENV_KEY: &'static str = "WAYLAND_DISPLAY";
    const FALLBACK_PATH: &'static str = "./wayland-0";
    const XDG_KEY: &'static str = "XDG_RUNTIME_DIR";

    /// Resolve the unix socket path created by the wayland server.
    ///
    /// Resolution is similar to `libwayland`'s [wl_display_connect], which is done using the
    /// following strategy:
    ///
    /// If no query path is provided, the resolver tries to see if `$WAYLAND_DISPLAY` is set. If
    /// neither exist, `./wayland-0` is used as the fallback query path.
    ///
    /// Next, the resolver checks if the query path is relative or absolute. Absolute paths are
    /// returned immediately, whereas relative ones are prepended with `$XDG_RUNTIME_DIR`.
    ///
    /// # Errors
    ///
    /// If the query path is relative and `$XDG_RUNTIME_DIR` is not set.
    ///
    /// [wl_display_connect]: https://wayland.freedesktop.org/docs/html/apb.html#Client-classwl__display_1a37233bec2632b424ff447a4a2abe3c5d
    pub fn resolve(query_path: Option<&'a Path>) -> Result<Self, SocketPathError> {
        let path = query_path
            .map(Into::into)
            .or_else(|| std::env::var_os(Self::ENV_KEY).map(PathBuf::from).map(Cow::Owned))
            .unwrap_or(Path::new(Self::FALLBACK_PATH).into());

        if path.is_absolute() {
            return Ok(Self(path));
        }

        let Some(xdg_path) = std::env::var_os(Self::XDG_KEY).map(PathBuf::from) else {
            return Err(SocketPathError::XdgRuntimeNotFound);
        };

        Ok(Self(xdg_path.join(path).into()))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_provided_absolute() {
        prepare_env(None, None);

        let provided = Path::new("/absolute/provided");
        let actual = SocketPath::resolve(Some(provided)).unwrap();

        assert_eq!(provided, actual.as_ref());
    }

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_provided_relative() {
        prepare_env(Some("/tmp"), None);

        let actual = SocketPath::resolve(Some(Path::new("./provided"))).unwrap();
        let expected = Path::new("/tmp/./provided");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_env_absolute() {
        prepare_env(None, Some("/absolute/test"));

        let actual = SocketPath::resolve(None).unwrap();
        let expected = Path::new("/absolute/test");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_env_relative() {
        prepare_env(Some("/tmp"), Some("./test"));

        let actual = SocketPath::resolve(None).unwrap();
        let expected = Path::new("/tmp/./test");

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_fallback() {
        let prefix = "/tmp";

        prepare_env(Some(prefix), None);

        let actual = SocketPath::resolve(None).unwrap();
        let expected = PathBuf::from(prefix).join(SocketPath::FALLBACK_PATH);

        assert_eq!(expected, actual.as_ref());
    }

    #[test]
    #[serial_test::serial(socket_path)]
    fn resolve_no_xdg_runtime_error() {
        prepare_env(None, None);

        let error = SocketPath::resolve(None).unwrap_err();
        assert_matches!(error, SocketPathError::XdgRuntimeNotFound);
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
