//! Syntactically valid DNS name (host) with a port.

use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use core::marker::PhantomData;
use core::net::{IpAddr, SocketAddr};
use core::str::FromStr;
#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::net::ToSocketAddrs;

use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A syntactically valid DNS name (host) with a port.
///
/// ## Notes
///
/// All IDN domain names are considered invalid for now and should be punycode
/// encoded before being parsed.
///
/// <div class=warning>
///
/// Currently, the lifetime parameter `'a` is not actually used and the
/// inner string type is `Arc<str>`. We will change the inner string type to
/// a more flexible one in the future, which accepts any borrowed, inlined
/// or owned ref-counted strings.
///
/// </div>
pub struct HostAddr<'a> {
    host: Host<'a>,
    port: u16,
    resolved: Option<IpAddr>,
}

impl<'a> HostAddr<'a> {
    /// Creates a new [`HostAddr`] from a host and a port number.
    pub fn new(host: &'a str, port: u16) -> Result<Self, ParseError> {
        Ok(Self {
            host: Host::new(host)?,
            port,
            resolved: None,
        })
    }

    #[allow(clippy::should_implement_trait, reason = "For lifetime stuff.")]
    /// Creates a new [`HostAddr`] from a string slice in the format of
    /// "host:port".
    ///
    /// ## Notes
    ///
    /// `IP:port` is not considered a valid format for now. Use
    /// [`UniAddr::from_str`] instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::host::HostAddr;
    ///
    /// let addr = HostAddr::from_str("example.com:8080").unwrap();
    /// assert_eq!(addr.host().as_ref(), "example.com");
    /// assert_eq!(addr.port(), 8080);
    /// assert_eq!(addr.resolved(), None);
    /// #
    /// # let _ = HostAddr::from_str("127.0.0.1:168").unwrap_err();
    /// # let _ = HostAddr::from_str("[::1]:168").unwrap_err();
    /// # let _ = HostAddr::from_str(":168").unwrap_err();
    /// # let _ = HostAddr::from_str("missing-port.com").unwrap_err();
    /// # let _ = HostAddr::from_str("missing-port.com:").unwrap_err();
    /// # let _ = HostAddr::from_str("invalid-port.com:168168").unwrap_err();
    /// # let _ = HostAddr::from_str("IDN域名.com:80").unwrap_err();
    /// ```
    ///
    /// [`UniAddr::from_str`]: crate::UniAddr::from_str
    pub fn from_str(s: &'a str) -> Result<Self, ParseError> {
        let mut parts = memchr::Memchr::new(b':', s.as_bytes()).rev();

        #[allow(clippy::string_slice, reason = "XXX")]
        let (host, port) = parts
            .next()
            .map(|idx| (&s[..idx], &s[idx + 1..]))
            .ok_or(ParseError::InvalidPort)?;

        let port = port.parse().map_err(|_| ParseError::InvalidPort)?;

        Self::new(host, port)
    }

    /// Converts this [`HostAddr`] into an owned version.
    ///
    /// This is a no-op for now since the inner string type is already owned,
    /// but it will be useful in the future when we change the inner string type
    /// to a more flexible one and accept borrowed strings.
    pub fn to_owned(self) -> HostAddr<'static> {
        HostAddr {
            host: self.host.to_owned(),
            port: self.port,
            resolved: self.resolved,
        }
    }

    #[inline]
    /// Returns a reference to the host.
    pub const fn host(&self) -> &Host<'a> {
        &self.host
    }

    #[inline]
    /// Returns the port number.
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[inline]
    /// Returns the resolved socket address.
    pub const fn resolved(&self) -> Option<SocketAddr> {
        match self.resolved {
            Some(ip) => Some(SocketAddr::new(ip, self.port)),
            None => None,
        }
    }

    #[cfg(feature = "std")]
    /// Resolves the host.
    ///
    /// By default, we utilize the method [`ToSocketAddrs::to_socket_addrs`]
    /// provided by the standard library to perform DNS resolution, which is a
    /// **blocking** operation and may take an arbitrary amount of time to
    /// complete, use with caution when called in asynchronous contexts.
    ///
    /// # Errors
    ///
    /// Resolution failure, or if no socket address resolved.
    pub fn blocking_resolve(&mut self) -> io::Result<()> {
        self.blocking_resolve_with(|host| {
            (host, 0)
                .to_socket_addrs()
                .and_then(|mut iter| {
                    iter.next()
                        .ok_or_else(|| io::Error::other("no socket address resolved"))
                })
                .map(|addr| addr.ip())
        })
    }

    /// Resolves the host with the given custom resolver function.
    ///
    /// The resolver function should take the host as input and return a
    /// `IpAddr` on success.
    ///
    /// # Errors
    ///
    /// Resolution failure, or if no socket address resolved.
    pub fn blocking_resolve_with<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&str) -> Result<IpAddr, E>,
    {
        let addr = f(&self.host)?;

        self.resolved = Some(addr);

        Ok(())
    }

    #[cfg(feature = "tokio")]
    /// Resolves the host asynchronously.
    ///
    /// It's highly recommended to use this method instead of the blocking
    /// version in asynchronous contexts.
    ///
    /// # Errors
    ///
    /// Resolution failure, or if no socket address resolved.
    pub async fn resolve(&mut self) -> io::Result<()> {
        self.resolve_with(|host| async move {
            tokio::net::lookup_host((host, 0))
                .await?
                .next()
                .map_or_else(
                    || Err(io::Error::other("no socket address resolved")),
                    |addr| Ok(addr.ip()),
                )
        })
        .await
    }

    /// Resolves the host asynchronously with the given custom resolver
    /// function.
    ///
    /// The resolver function should take the host as input and return a
    /// `IpAddr` on success.
    ///
    /// # Errors
    ///
    /// Resolution failure, or if no socket address resolved.
    pub async fn resolve_with<'fut, F, Fut, E>(&'fut mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&'fut str) -> Fut + Send,
        Fut: Future<Output = Result<IpAddr, E>> + Send + 'fut,
    {
        let addr = f(&self.host).await?;

        self.resolved = Some(addr);

        Ok(())
    }
}

impl fmt::Display for HostAddr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl FromStr for HostAddr<'static> {
    type Err = ParseError;

    /// See [`HostAddr::from_str`] for details.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HostAddr::from_str(s).map(HostAddr::to_owned)
    }
}

wrapper_lite::wrapper!(
    #[wrapper_impl(Debug)]
    #[wrapper_impl(Display)]
    #[wrapper_impl(AsRef<str>)]
    #[wrapper_impl(Deref<str>)]
    #[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
    /// A syntactically valid DNS name.
    ///
    /// <div class=warning>
    ///
    /// Currently, the lifetime parameter `'a` is not actually used and the
    /// inner string type is `Arc<str>`. We will change the inner string type to
    /// a more flexible one in the future, which accepts any borrowed, inlined
    /// or owned ref-counted strings.
    ///
    /// </div>
    pub struct Host<'a> {
        inner: Arc<str>,
        _lt_placeholder: PhantomData<&'a ()>,
    }
);

impl<'a> Host<'a> {
    /// Creates a new `Host` from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the input string is not a valid DNS name.
    pub fn new(host: &'a str) -> Result<Self, ParseError> {
        if Self::validate_host_name(host.as_bytes()).is_ok() {
            Ok(Self {
                inner: host.into(),
                _lt_placeholder: PhantomData,
            })
        } else {
            Err(ParseError::InvalidHost)
        }
    }

    /// Converts this [`Host`] into an owned version.
    ///
    /// This is a no-op for now since the inner string type is already owned,
    /// but it will be useful in the future when we change the inner string type
    /// to a more flexible one and accept borrowed strings.
    pub fn to_owned(self) -> Host<'static> {
        Host {
            inner: self.inner,
            _lt_placeholder: PhantomData,
        }
    }

    // https://github.com/rustls/pki-types/blob/b8c04aa6b7a34875e2c4a33edc9b78d31da49523/src/server_name.rs
    const fn validate_host_name(input: &[u8]) -> Result<(), ()> {
        enum State {
            Start,
            Next,
            NumericOnly { len: usize },
            NextAfterNumericOnly,
            Subsequent { len: usize },
            Hyphen { len: usize },
        }

        /// "Labels must be 63 characters or less."
        const MAX_LABEL_LENGTH: usize = 63;

        /// <https://devblogs.microsoft.com/oldnewthing/20120412-00/?p=7873>
        const MAX_NAME_LENGTH: usize = 253;

        let mut state = State::Start;

        if input.len() > MAX_NAME_LENGTH {
            return Err(());
        }

        let mut idx = 0;
        while idx < input.len() {
            let ch = input[idx];
            state = match (state, ch) {
                (
                    State::Start | State::Next | State::NextAfterNumericOnly | State::Hyphen { .. },
                    b'.',
                ) => {
                    return Err(());
                }
                (State::Subsequent { .. }, b'.') => State::Next,
                (State::NumericOnly { .. }, b'.') => State::NextAfterNumericOnly,
                (
                    State::Subsequent { len } | State::NumericOnly { len } | State::Hyphen { len },
                    _,
                ) if len >= MAX_LABEL_LENGTH => {
                    return Err(());
                }
                (State::Start | State::Next | State::NextAfterNumericOnly, b'0'..=b'9') => {
                    State::NumericOnly { len: 1 }
                }
                (State::NumericOnly { len }, b'0'..=b'9') => State::NumericOnly { len: len + 1 },
                (
                    State::Start | State::Next | State::NextAfterNumericOnly,
                    b'a'..=b'z' | b'A'..=b'Z' | b'_',
                ) => State::Subsequent { len: 1 },
                (
                    State::Subsequent { len } | State::NumericOnly { len } | State::Hyphen { len },
                    b'-',
                ) => State::Hyphen { len: len + 1 },
                (
                    State::Subsequent { len } | State::NumericOnly { len } | State::Hyphen { len },
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9',
                ) => State::Subsequent { len: len + 1 },
                _ => return Err(()),
            };
            idx += 1;
        }

        if matches!(
            state,
            State::Start
                | State::Hyphen { .. }
                | State::NumericOnly { .. }
                | State::NextAfterNumericOnly
                | State::Next
        ) {
            return Err(());
        }

        Ok(())
    }
}
