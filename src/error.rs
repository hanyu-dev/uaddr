//! Error types for `uni-addr`.

use core::fmt;
use std::io;

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

    /// Invalid UDS address format
    InvalidUDSAddress(io::Error),

    /// Unsupported address type on this platform
    Unsupported,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty address string"),
            Self::InvalidHost => write!(f, "invalid host name"),
            Self::InvalidPort => write!(f, "invalid port"),
            Self::InvalidUDSAddress(err) => write!(f, "invalid UDS address: {err}"),
            Self::Unsupported => write!(f, "unsupported address type on this platform"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUDSAddress(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ParseError> for io::Error {
    fn from(value: ParseError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}
