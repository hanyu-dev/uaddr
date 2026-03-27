//! Error types for `uaddr`.

use core::fmt;

#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(debug_assertions, derive(PartialEq, Eq))]
/// Errors that can occur when parsing a [`UniAddr`] / [`HostAddr`] /
/// [`UnixAddr`] from a string.
///
/// [`UniAddr`]: crate::UniAddr
/// [`HostAddr`]: crate::host::HostAddr
/// [`UnixAddr`]: crate::unix::UnixAddr
pub enum ParseError {
    /// Expected an non-empty string / bytes, but got an empty one.
    Empty,

    /// Invalid host, parsing error or does not exist.
    InvalidHost,

    /// Invalid port, parsing error or does not exist.
    InvalidPort,

    /// Invalid [`UnixAddr`].
    /// 
    /// [`UnixAddr`]: crate::unix::UnixAddr
    InvalidUnixAddr,

    /// Unsupported address type for this operation on the current platform.
    Unsupported,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "invalid input: expecting a non-empty string / bytes"),
            Self::InvalidHost => write!(f, "invalid host"),
            Self::InvalidPort => write!(f, "invalid port"),
            Self::InvalidUnixAddr => write!(f, "invalid UNIX domain socket address"),
            Self::Unsupported => write!(
                f,
                "unsupported address type for this operation on the current platform"
            ),
        }
    }
}

impl core::error::Error for ParseError {}

#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(debug_assertions, derive(PartialEq, Eq))]
/// An error type indicating that the [`UniAddr`] is invalid for some reason
///
/// [`UniAddr`]: crate::UniAddr
pub enum InvalidUniAddr {
    /// The [`HostAddr`] is unresolved.
    /// 
    /// [`HostAddr`]: crate::host::HostAddr
    Unresolved,

    /// The [`UniAddr`] is a [`UnixAddr`], which cannot be converted to a
    /// [`SocketAddr`], etc.
    ///
    /// [`UniAddr`]: crate::UniAddr
    /// [`HostAddr`]: crate::host::HostAddr
    /// [`UnixAddr`]: crate::unix::UnixAddr
    /// [`SocketAddr`]: std::net::SocketAddr
    Unsupported,
}

impl fmt::Display for InvalidUniAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unresolved => write!(f, "unresolved host address"),
            Self::Unsupported => write!(
                f,
                "unsupported address type for this operation on the current platform"
            ),
        }
    }
}

impl core::error::Error for InvalidUniAddr {}
