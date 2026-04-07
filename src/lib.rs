#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use core::future::Future;
use core::net::SocketAddr;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::io;

pub use crate::error::{InvalidUniAddr, ParseError};
pub use crate::host::HostAddr;
pub use crate::unix::{UnixAddr, SUN_LEN, UNIX_PREFIX, UNIX_URI_PREFIX};

mod bridge;
mod error;
mod host;
mod unix;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A unified address type that can represent:
///
/// - [`SocketAddr`];
/// - [`UnixAddr`];
/// - [`HostAddr`].
///
/// <div class=warning>
///
/// Currently, the lifetime parameter `'a` is not actually used. See
/// [`UnixAddr`] or [`HostAddr`] for more details.
///
/// </div>
pub enum UniAddr<'a> {
    /// [`SocketAddr`].
    Inet(SocketAddr),

    /// [`UnixAddr`].
    Unix(UnixAddr<'a>),

    /// [`HostAddr`].
    Host(HostAddr<'a>),
}

impl<'a> UniAddr<'a> {
    #[allow(clippy::should_implement_trait, reason = "For lifetime stuff.")]
    /// Creates a new [`UniAddr`] from the given string representation.
    ///
    /// ```rust
    /// use uaddr::UniAddr;
    ///
    /// // An IPv4 address with a port
    /// let _ = UniAddr::from_str("127.0.0.1:13168").unwrap();
    /// // An IPv6 address with a port
    /// let _ = UniAddr::from_str("[::1]:13168").unwrap();
    /// // A host with a port
    /// let _ = UniAddr::from_str("example.com:8080").unwrap();
    /// // A UDS address (UNIX domain socket)
    /// let _ = UniAddr::from_str("unix:/path/to/your/file.socket").unwrap();
    /// // A URI-style UDS address (not recommended)
    /// let _ = UniAddr::from_str("unix:///path/to/your/file.socket").unwrap();
    /// // An abstract namespace UDS address.
    /// let _ = UniAddr::from_str("unix:@abstract-socket").unwrap();
    /// // A URI-style abstract namespace UDS address (not recommended).
    /// let _ = UniAddr::from_str("unix://@abstract-socket").unwrap();
    /// ```
    pub fn from_str(string: &'a str) -> Result<Self, ParseError> {
        if string.starts_with(UNIX_URI_PREFIX) {
            return UnixAddr::from_str(string).map(UniAddr::Unix);
        }

        if string.starts_with(UNIX_PREFIX) {
            return UnixAddr::from_str(string).map(UniAddr::Unix);
        }

        if let Ok(addr) = SocketAddr::from_str(string) {
            return Ok(UniAddr::Inet(addr));
        }

        HostAddr::from_str(string).map(UniAddr::Host)
    }

    /// Returns the resolved socket address if this is an [`SocketAddr`] or a
    /// resolved [`HostAddr`].
    ///
    /// ```rust
    /// use uaddr::UniAddr;
    ///
    /// // An already resolved IP address with a port can be resolved to a socket address.
    /// let mut addr = UniAddr::from_str("1.1.1.1:443").unwrap();
    /// assert!(addr
    ///     .resolved()
    ///     .is_ok_and(|addr| addr == "1.1.1.1:443".parse().unwrap()));
    /// // An unresolved host address cannot be resolved to a socket address.
    /// let mut addr = UniAddr::from_str("example.com:8080").unwrap();
    /// assert!(addr.resolved().is_err());
    /// # if addr.blocking_resolve_host_name().is_ok() {
    /// #     assert!(addr.resolved().is_ok());
    /// # }
    /// // One UNIX domain socket address cannot be resolved to a socket address.
    /// let mut addr = UniAddr::from_str("unix:/path/to/your/file.socket").unwrap();
    /// assert!(addr.resolved().is_err());
    /// ```
    pub const fn resolved(&self) -> Result<SocketAddr, InvalidUniAddr> {
        match self {
            Self::Inet(addr) => Ok(*addr),
            Self::Unix(_) => Err(InvalidUniAddr::Unsupported),
            Self::Host(host) => match host.resolved() {
                Some(addr) => Ok(addr),
                None => Err(InvalidUniAddr::Unresolved),
            },
        }
    }

    #[cfg(feature = "std")]
    /// Resolves the hostname if this is a [`HostAddr`].
    ///
    /// This is a no-op for `Inet` and  `Unix` variants.
    ///
    /// See [`HostAddr::blocking_resolve`] for details and error semantics.
    pub fn blocking_resolve_host_name(&mut self) -> io::Result<()> {
        match self {
            Self::Host(addr) => addr.blocking_resolve(),
            _ => Ok(()),
        }
    }

    /// Resolves the hostname using a custom resolver if this is a [`HostAddr`].
    ///
    /// This is a no-op for `Inet` and `Unix` variants.
    ///
    /// See [`HostAddr::blocking_resolve_with`] for details.
    pub fn blocking_resolve_host_name_with<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&str) -> Result<SocketAddr, E>,
    {
        match self {
            Self::Host(addr) => addr.blocking_resolve_with(f),
            _ => Ok(()),
        }
    }

    #[cfg(feature = "tokio")]
    /// Resolves the hostname asynchronously if this is a [`HostAddr`].
    ///
    /// This is a no-op for `Inet` and `Unix` variants.
    ///
    /// See [`HostAddr::resolve`] for details and error semantics.
    pub async fn resolve_host_name(&mut self) -> io::Result<()> {
        match self {
            Self::Host(addr) => addr.resolve().await,
            _ => Ok(()),
        }
    }

    /// Resolves the hostname asynchronously using a custom resolver if this is
    /// a [`HostAddr`].
    ///
    /// This is a no-op for `Inet` and `Unix` variants.
    ///
    /// See [`HostAddr::resolve_with`] for details.
    pub async fn resolve_host_name_with<'fut, F, Fut, E>(&'fut mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&'fut str) -> Fut + Send,
        Fut: Future<Output = Result<SocketAddr, E>> + Send + 'fut,
    {
        match self {
            Self::Host(addr) => addr.resolve_with(f).await,
            _ => Ok(()),
        }
    }

    /// Converts this [`UniAddr`] into an owned version.
    ///
    /// This is a no-op for now since the inner bytes type is already owned,
    /// but it will be useful in the future when we change the inner bytes type
    /// to a more flexible one and accept borrowed bytes.
    pub fn to_owned(self) -> UniAddr<'static> {
        match self {
            Self::Inet(addr) => UniAddr::Inet(addr),
            Self::Unix(addr) => UniAddr::Unix(addr.to_owned()),
            Self::Host(addr) => UniAddr::Host(addr.to_owned()),
        }
    }
}

impl fmt::Display for UniAddr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inet(addr) => addr.fmt(f),
            Self::Unix(addr) => addr.fmt(f),
            Self::Host(host) => host.fmt(f),
        }
    }
}

impl FromStr for UniAddr<'static> {
    type Err = ParseError;

    /// See [`Self::from_str`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UniAddr::from_str(s).map(UniAddr::to_owned)
    }
}

impl From<SocketAddr> for UniAddr<'_> {
    fn from(addr: SocketAddr) -> Self {
        UniAddr::Inet(addr)
    }
}

impl From<&SocketAddr> for UniAddr<'_> {
    fn from(addr: &SocketAddr) -> Self {
        UniAddr::Inet(*addr)
    }
}

impl TryFrom<UniAddr<'_>> for SocketAddr {
    type Error = InvalidUniAddr;

    /// See [`UniAddr::resolved`].
    fn try_from(value: UniAddr<'_>) -> Result<Self, Self::Error> {
        value.resolved()
    }
}

impl TryFrom<&UniAddr<'_>> for SocketAddr {
    type Error = InvalidUniAddr;

    /// See [`UniAddr::resolved`].
    fn try_from(value: &UniAddr<'_>) -> Result<Self, Self::Error> {
        value.resolved()
    }
}

impl<'a> From<HostAddr<'a>> for UniAddr<'a> {
    fn from(addr: HostAddr<'a>) -> Self {
        UniAddr::Host(addr)
    }
}

impl<'a> From<&HostAddr<'a>> for UniAddr<'a> {
    fn from(addr: &HostAddr<'a>) -> Self {
        UniAddr::Host(addr.clone())
    }
}

impl<'a> From<UnixAddr<'a>> for UniAddr<'a> {
    fn from(addr: UnixAddr<'a>) -> Self {
        UniAddr::Unix(addr)
    }
}

impl<'a> From<&UnixAddr<'a>> for UniAddr<'a> {
    fn from(addr: &UnixAddr<'a>) -> Self {
        UniAddr::Unix(addr.clone())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString as _;

    use super::*;

    #[test]
    fn smoking() {
        macro_rules! test {
            ($text:expr) => {{
                assert_eq!(UniAddr::from_str($text).unwrap().to_string(), $text);
            }};
            ($text:expr, $expected:expr) => {{
                assert_eq!(UniAddr::from_str($text).unwrap().to_string(), $expected);
            }};
            (F $text:expr) => {{
                let _ = UniAddr::from_str($text).unwrap_err();
            }};
        }

        test!("127.0.0.1:13168");
        test!("[::1]:13168");

        test!("example.com:8080");

        test!("unix:/path/to/your/file.socket");
        test!(
            "unix:///path/to/your/file.socket",
            "unix:/path/to/your/file.socket"
        );
        test!("unix:@abstract-socket");
        test!("unix://@abstract-socket", "unix:@abstract-socket");
    }
}
