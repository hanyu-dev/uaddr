//! Error types for `uni-addr`.

use core::fmt;

#[derive(Debug)]
/// Errors that can occur when parsing a [`UniAddr`] from a string.
///
/// [`UniAddr`]: crate::UniAddr
pub enum ParseError {
    /// Empty input string
    Empty,

    /// Invalid or missing hostname, or an invalid Ipv4 / IPv6 address
    InvalidHost,

    /// Invalid address format: missing or invalid port
    InvalidPort,

    #[cfg(all(unix, any(test, feature = "std")))]
    /// Invalid UDS address format
    InvalidUDSAddress(std::io::Error),

    /// Unsupported address type on this platform
    Unsupported,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty address string"),
            Self::InvalidHost => write!(f, "invalid host name"),
            Self::InvalidPort => write!(f, "invalid port"),
            #[cfg(all(unix, any(test, feature = "std")))]
            Self::InvalidUDSAddress(err) => write!(f, "invalid UDS address: {err}"),
            Self::Unsupported => write!(f, "unsupported address type on this platform"),
        }
    }
}

impl core::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            #[cfg(all(unix, any(test, feature = "std")))]
            Self::InvalidUDSAddress(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl From<ParseError> for std::io::Error {
    fn from(value: ParseError) -> Self {
        std::io::Error::other(value)
    }
}
