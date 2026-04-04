//! UNIX domain socket (UDS) address.

use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::str::FromStr;

use crate::error::ParseError;

/// Prefix for UDS addresses in URI format.
pub const UNIX_URI_PREFIX: &str = "unix://";

/// Prefix for UDS addresses in general format.
pub const UNIX_PREFIX: &str = "unix:";

wrapper_lite::wrapper!(
    #[gen(AsRef<[u8]>)]
    #[derive(Clone, PartialEq, Eq, Hash)]
    /// A UNIX domain socket (UDS) address.
    ///
    /// Three types of address are distinguished:
    ///
    /// 1. `pathname`: a non-empty bytes slice without any interior null bytes.
    /// 1. `unnamed`: an empty bytes slice.
    /// 1. `abstract`: a bytes slice that starts with `b'\0'`.
    ///
    /// The maximum length of the address bytes is `SUN_LEN`, which is 108 on
    /// most Unix-like platforms.
    ///
    /// ## Notes
    ///
    /// It should be noted that the abstract namespace is a
    /// Linux-specific extension. While creating an abstract address
    /// on other platforms is allowed, converting an [`UnixAddr`] to the
    /// standard library type [`SocketAddr`] is a hard error (compilation
    /// error).
    ///
    /// Additionally, any bytes slice that starts with `b'\0'` is a valid
    /// abstract address, which means that an abstract address with interior
    /// null bytes or even an empty abstract address is a "legal" abstract
    /// address. Such address may lead to some unexpected behaviors and is
    /// rejected here by default. You can use the [`from_abstract_name`] method
    /// with `LOOSE_MODE` set to true to manually construct such abstract
    /// addresses if you really need them.
    ///
    /// <div class=warning>
    ///
    /// Currently, the lifetime parameter `'a` is not actually used and the
    /// inner bytes type is `Arc<[u8]>`. We will change the inner bytes type to
    /// a more flexible one in the future, which accepts any borrowed, inlined
    /// or owned atomic-ref-counted bytes.
    ///
    /// </div>
    ///
    /// [`SocketAddr`]: std::os::unix::net::SocketAddr
    /// [`from_abstract_name`]: Self::from_abstract_name
    pub struct UnixAddr<'a> {
        bytes: Arc<[u8]>,
        #[default(PhantomData)]
        _lt_placeholder: PhantomData<&'a ()>,
    }
);

#[cfg(unix)]
const SUN_LEN: usize =
    core::mem::size_of::<libc::sockaddr_un>() - core::mem::size_of::<libc::sa_family_t>();

#[cfg(not(unix))]
const SUN_LEN: usize = usize::MAX;

impl<'a> UnixAddr<'a> {
    #[allow(clippy::should_implement_trait, reason = "For lifetime stuff.")]
    /// Parses (deserializes) the given string to a [`UnixAddr`].
    ///
    /// This method accepts the following two serialization formats:
    ///
    /// 1. `unix:{unix-socket-address}`;
    /// 1. `unix://{unix-socket-address}`.
    ///
    /// One `{unix-socket-address}` may be a file system path (for pathname
    /// addresses), or a string starting with `@` or `\0` (for abstract
    /// addresses), or an empty string (for unnamed addresses).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    /// #
    /// # let _ = UnixAddr::from_str("/some/path/without/unix/prefix").unwrap_err();
    ///
    /// let addr = UnixAddr::from_str("unix:/path/to/your/file.socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"/path/to/your/file.socket"[..]));
    ///
    /// let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let addr = UnixAddr::from_str("unix:\0abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// // By default, we don't accept abstract socket names with interior null bytes.
    /// let _ = UnixAddr::from_str("unix:@abstract-socket\0").unwrap_err();
    /// # let _ = UnixAddr::from_str("unix:\0abstract-socket\0").unwrap_err();
    ///
    /// let addr = UnixAddr::from_str("unix:").unwrap();
    /// assert!(addr.is_unnamed());
    /// ```
    pub fn from_str(string: &'a str) -> Result<Self, ParseError> {
        let Some(string) = string
            .strip_prefix(UNIX_URI_PREFIX)
            .or_else(|| string.strip_prefix(UNIX_PREFIX))
        else {
            return Err(ParseError::InvalidUnixAddr);
        };

        let bytes = string.as_bytes();

        match bytes {
            [b'\0' | b'@', bytes @ ..] => Self::from_abstract_name::<false>(bytes),
            bytes @ [_, ..] => Self::from_pathname(bytes),
            [] => Ok(Self::new_unnamed()),
        }
    }

    /// Creates a new [`UnixAddr`] directly from the given bytes slice.
    ///
    /// ## Notes
    ///
    /// 1. `@` is a valid character for pathname. Unlike [`from_str`], `@` is
    ///    not treated as the indicator of an abstract UDS address here.
    /// 1. Unlike [`from_str`], we accept abstract socket names with interior
    ///    null bytes here.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_bytes(b"/path/to/your/file.socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"/path/to/your/file.socket"[..]));
    ///
    /// let addr = UnixAddr::from_bytes(b"@abstract-socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"@abstract-socket"[..]));
    ///
    /// // One pathname address with interior null bytes is invalid.
    /// let _ = UnixAddr::from_bytes(b"@abstract-socket\0").unwrap_err();
    ///
    /// let addr = UnixAddr::from_bytes(b"\0abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let addr = UnixAddr::from_bytes(b"\0abstract-socket\0").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket\0"[..]));
    ///
    /// let addr = UnixAddr::from_bytes(b"").unwrap();
    /// assert!(addr.is_unnamed());
    /// ```
    ///
    /// [`from_str`]: Self::from_str
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ParseError> {
        match bytes {
            bytes @ [b'\0', ..] => Self::from_abstract_name_bytes::<true>(bytes),
            bytes @ [_, ..] => Self::from_pathname(bytes),
            [] => Ok(Self::new_unnamed()),
        }
    }

    /// Creates a new [`UnixAddr`] from the given pathname.
    ///
    /// ## Notes
    ///
    /// `@` is a valid character for pathname, we just use it to replace `b'\0'`
    /// during serialization. Unlike [`from_str`], `@` is not treated as the
    /// indicator of an abstract UDS address here.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_pathname(b"/path/to/your/file.socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"/path/to/your/file.socket"[..]));
    ///
    /// // Note: this is a special case.
    /// let addr = UnixAddr::from_pathname(b"@abstract-socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"@abstract-socket"[..]));
    ///
    /// let _ = UnixAddr::from_pathname(b"\0abstract-socket").unwrap_err();
    /// let _ = UnixAddr::from_pathname(b"").unwrap_err();
    /// ```
    ///
    /// [`from_str`]: Self::from_str
    pub fn from_pathname(path: &'a [u8]) -> Result<Self, ParseError> {
        if path.is_empty() {
            return Err(ParseError::Empty);
        }

        if path.len() > SUN_LEN {
            return Err(ParseError::InvalidUnixAddr);
        }

        if memchr::memchr(b'\0', path).is_some() {
            return Err(ParseError::InvalidUnixAddr);
        }

        Ok(Self::from_inner(Arc::from(path)))
    }

    /// [`from_pathname`], but terminates the bytes at the first null byte.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_pathname_until_nul(b"/path/to/your/file.socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"/path/to/your/file.socket"[..]));
    ///
    /// let addr = UnixAddr::from_pathname_until_nul(b"/path/to/your/file.sock\0et\0").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"/path/to/your/file.sock"[..]));
    ///
    /// let addr = UnixAddr::from_pathname_until_nul(b"@abstract-socket").unwrap();
    /// assert!(addr.is_pathname());
    /// assert_eq!(addr.as_pathname(), Some(&b"@abstract-socket"[..]));
    ///
    /// let _ = UnixAddr::from_pathname_until_nul(b"").unwrap_err();
    /// let _ = UnixAddr::from_pathname_until_nul(b"\0").unwrap_err();
    /// ```
    ///
    /// [`from_pathname`]: Self::from_pathname
    pub fn from_pathname_until_nul(path: &'a [u8]) -> Result<Self, ParseError> {
        if path.is_empty() {
            return Err(ParseError::Empty);
        }

        let bytes;

        if path.len() > SUN_LEN {
            let Some(idx) = memchr::memchr(b'\0', &path[..SUN_LEN]) else {
                return Err(ParseError::InvalidUnixAddr);
            };

            bytes = &path[..idx];
        } else {
            match memchr::memchr(b'\0', path) {
                Some(idx) => bytes = &path[..idx],
                None => bytes = path,
            }
        }

        if bytes.is_empty() {
            return Err(ParseError::InvalidUnixAddr);
        }

        Ok(Self::from_inner(Arc::from(bytes)))
    }

    /// Checks if the address is a *pathname* one.
    pub fn is_pathname(&self) -> bool {
        !self.is_abstract_name() && !self.is_unnamed()
    }

    /// Returns the pathname bytes if this is a pathname address, or `None`
    /// otherwise.
    pub fn as_pathname(&self) -> Option<&[u8]> {
        if self.is_pathname() {
            Some(self.bytes.as_ref())
        } else {
            None
        }
    }

    /// Creates a new abstract [`UnixAddr`] from the given name.
    ///
    /// ## Notes
    ///
    /// Don't include the leading `b'\0'` in the name, as it will be
    /// automatically added by this method. If you already have a bytes slice
    /// that starts with `b'\0'`, use [`from_abstract_name_bytes`] instead.
    ///
    /// As mentioned in the documentation of this type, any bytes slice that
    /// starts with `b'\0'` is a valid abstract address, including those with
    /// interior null bytes or even an empty one. Such addresses may lead to
    /// some unexpected behaviors and are rejected by default. You can set
    /// `LOOSE_MODE` to true to manually construct such abstract addresses if
    /// you really need them.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_abstract_name::<false>(b"abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name::<false>(b"").unwrap_err();
    /// let addr = UnixAddr::from_abstract_name::<true>(b"").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b""[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name::<false>(b"abstract\0socket").unwrap_err();
    /// let addr = UnixAddr::from_abstract_name::<true>(b"abstract\0socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract\0socket"[..]));
    /// ```
    ///
    /// [`from_abstract_name_bytes`]: Self::from_abstract_name_bytes
    pub fn from_abstract_name<const LOOSE_MODE: bool>(name: &'a [u8]) -> Result<Self, ParseError> {
        if name.len() > SUN_LEN - 1 {
            return Err(ParseError::InvalidUnixAddr);
        }

        if !LOOSE_MODE {
            if name.is_empty() {
                return Err(ParseError::Empty);
            }

            if memchr::memchr(b'\0', name).is_some() {
                return Err(ParseError::InvalidUnixAddr);
            }
        }

        Ok(Self::from_abstract_name_unchecked(name))
    }

    /// Creates a new abstract [`UnixAddr`] from its bytes representation, i.e.,
    /// a bytes slice that starts with `b'\0'`.
    ///
    /// ## Notes
    ///
    /// As mentioned in the documentation of this type, any bytes slice that
    /// starts with `b'\0'` is a valid abstract address, including those with
    /// interior null bytes or even an empty one. Such addresses may lead to
    /// some unexpected behaviors and are rejected by default. You can set
    /// `LOOSE_MODE` to true to manually construct such abstract addresses if
    /// you really need them.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes::<false>(b"\0abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name_bytes::<false>(b"\0").unwrap_err();
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes::<true>(b"\0").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b""[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name_bytes::<false>(b"\0abstract\0socket").unwrap_err();
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes::<true>(b"\0abstract\0socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract\0socket"[..]));
    /// ```
    pub fn from_abstract_name_bytes<const LOOSE_MODE: bool>(
        bytes: &'a [u8],
    ) -> Result<Self, ParseError> {
        if bytes.len() > SUN_LEN {
            return Err(ParseError::InvalidUnixAddr);
        }

        if bytes.is_empty() || bytes[0] != b'\0' {
            return Err(ParseError::InvalidUnixAddr);
        }

        if !LOOSE_MODE {
            if bytes[1..].is_empty() {
                return Err(ParseError::Empty);
            }

            if memchr::memchr(b'\0', &bytes[1..]).is_some() {
                return Err(ParseError::InvalidUnixAddr);
            }
        }

        Ok(Self::from_abstract_name_bytes_unchecked(bytes))
    }

    /// [`from_abstract_name`](Self::from_abstract_name), but terminates the
    /// name at the first null byte.
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_abstract_name_until_nul::<false>(b"abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let addr = UnixAddr::from_abstract_name_until_nul::<false>(b"abstract\0socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract"[..]));
    /// #
    /// # let addr = UnixAddr::from_abstract_name_until_nul::<true>(b"abstract\0socket").unwrap();
    /// # assert!(addr.is_abstract_name());
    /// # assert_eq!(addr.as_abstract_name(), Some(&b"abstract"[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name_until_nul::<false>(b"").unwrap_err();
    /// let addr = UnixAddr::from_abstract_name_until_nul::<true>(b"").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b""[..]));
    /// ```
    pub fn from_abstract_name_until_nul<const LOOSE_MODE: bool>(
        name: &'a [u8],
    ) -> Result<Self, ParseError> {
        let bytes;

        if name.len() > SUN_LEN - 1 {
            let Some(idx) = memchr::memchr(b'\0', &name[..SUN_LEN]) else {
                return Err(ParseError::InvalidUnixAddr);
            };

            bytes = &name[..idx];
        } else {
            match memchr::memchr(b'\0', name) {
                Some(idx) => bytes = &name[..idx],
                None => bytes = name,
            }
        }

        #[allow(clippy::collapsible_if, reason = "XXX")]
        if !LOOSE_MODE {
            if bytes.is_empty() {
                return Err(ParseError::Empty);
            }
        }

        Ok(Self::from_abstract_name_unchecked(bytes))
    }

    /// [`from_abstract_name_bytes`], but terminates the name at the first null
    /// byte.
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes_until_nul::<false>(b"\0abstract-socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes_until_nul::<false>(b"\0abstract\0socket").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract"[..]));
    /// #
    /// # let addr = UnixAddr::from_abstract_name_bytes_until_nul::<true>(b"\0abstract\0socket").unwrap();
    /// # assert!(addr.is_abstract_name());
    /// # assert_eq!(addr.as_abstract_name(), Some(&b"abstract"[..]));
    ///
    /// let _ = UnixAddr::from_abstract_name_bytes_until_nul::<false>(b"\0").unwrap_err();
    ///
    /// let addr = UnixAddr::from_abstract_name_bytes_until_nul::<true>(b"\0").unwrap();
    /// assert!(addr.is_abstract_name());
    /// assert_eq!(addr.as_abstract_name(), Some(&b""[..]));
    /// ```
    ///
    /// [`from_abstract_name_bytes`]: Self::from_abstract_name_bytes
    pub fn from_abstract_name_bytes_until_nul<const LOOSE_MODE: bool>(
        mut bytes: &'a [u8],
    ) -> Result<Self, ParseError> {
        if bytes.is_empty() || bytes[0] != b'\0' {
            return Err(ParseError::InvalidUnixAddr);
        }

        if bytes.len() > SUN_LEN {
            let Some(idx) = memchr::memchr(b'\0', &bytes[1..=SUN_LEN]) else {
                return Err(ParseError::InvalidUnixAddr);
            };

            bytes = &bytes[..=idx];
        } else if let Some(idx) = memchr::memchr(b'\0', &bytes[1..]) {
            bytes = &bytes[..=idx];
        } else {
            // nothing to do.
        }

        #[allow(clippy::collapsible_if, reason = "XXX")]
        if !LOOSE_MODE {
            if bytes.len() == b"\0".len() {
                return Err(ParseError::Empty);
            }
        }

        Ok(Self::from_abstract_name_bytes_unchecked(bytes))
    }

    fn from_abstract_name_unchecked(name: &'a [u8]) -> Self {
        let mut bytes: Arc<[MaybeUninit<u8>]> = Arc::new_uninit_slice(name.len() + 1);

        #[allow(unsafe_code, reason = "XXX")]
        unsafe {
            let ptr = Arc::make_mut(&mut bytes).as_mut_ptr();

            ptr.write(MaybeUninit::new(b'\0'));
            ptr.add(1)
                .copy_from_nonoverlapping(name.as_ptr().cast(), name.len());
        }

        #[allow(unsafe_code, reason = "XXX")]
        let bytes = unsafe { bytes.assume_init() };

        Self::from_inner(bytes)
    }

    fn from_abstract_name_bytes_unchecked(bytes: &'a [u8]) -> Self {
        Self::from_inner(bytes.into())
    }

    /// Checks if the UDS address is an *abstract* one.
    pub fn is_abstract_name(&self) -> bool {
        self.bytes.first().is_some_and(|b| *b == b'\0')
    }

    /// Returns the abstract name if this is an abstract UDS address, or
    /// `None` otherwise.
    ///
    /// The returned bytes slice does not include the leading `b'\0'`.
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
    ///
    /// assert_eq!(addr.as_abstract_name(), Some(&b"abstract-socket"[..]));
    /// ```
    pub fn as_abstract_name(&self) -> Option<&[u8]> {
        if self.is_abstract_name() {
            Some(&self.bytes.as_ref()[1..])
        } else {
            None
        }
    }

    /// Returns the abstract name bytes if this is an abstract UDS address, or
    /// `None` otherwise.
    ///
    /// The returned bytes slice includes the leading `b'\0'`.
    ///
    /// ```rust
    /// use uaddr::unix::UnixAddr;
    ///
    /// let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
    ///
    /// assert_eq!(
    ///     addr.as_abstract_name_bytes(),
    ///     Some(&b"\0abstract-socket"[..])
    /// );
    /// ```
    pub fn as_abstract_name_bytes(&self) -> Option<&[u8]> {
        if self.is_abstract_name() {
            Some(self.bytes.as_ref())
        } else {
            None
        }
    }

    /// Creates an new unnamed [`UnixAddr`].
    pub fn new_unnamed() -> Self {
        Self::from_inner(Arc::from([]))
    }

    /// Checks if the UDS address is an *unnamed* one.
    pub fn is_unnamed(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Converts this [`UnixAddr`] into an owned version.
    ///
    /// This is a no-op for now since the inner bytes type is already owned,
    /// but it will be useful in the future when we change the inner bytes type
    /// to a more flexible one and accept borrowed bytes.
    pub fn to_owned(self) -> UnixAddr<'static> {
        UnixAddr::from_inner(self.bytes)
    }
}

impl fmt::Debug for UnixAddr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(self.bytes.len());

        if let Some(pathname) = self.as_pathname() {
            for u in pathname.utf8_chunks() {
                buf.push_str(u.valid());

                for b in u.invalid() {
                    buf.push_str("\\x");
                    buf.push_str(itoa::Buffer::new().format(*b));
                }
            }
        } else if let Some(abstract_name) = self.as_abstract_name() {
            buf.push('@');

            for u in abstract_name.utf8_chunks() {
                buf.push_str(u.valid());

                for b in u.invalid() {
                    buf.push_str("\\x");
                    buf.push_str(itoa::Buffer::new().format(*b));
                }
            }
        } else if self.is_unnamed() {
            buf.push_str("unnamed");
        } else {
            // unreachable.
        }

        f.debug_tuple("UnixAddr").field(&buf).finish()
    }
}

impl fmt::Display for UnixAddr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::with_capacity(self.bytes.len());

        buf.push_str(UNIX_PREFIX);

        if let Some(pathname) = self.as_pathname() {
            for u in pathname.utf8_chunks() {
                buf.push_str(u.valid());

                if !u.invalid().is_empty() {
                    buf.push('\u{FFFD}');
                }
            }
        } else if let Some(abstract_name) = self.as_abstract_name() {
            buf.push('@');

            for u in abstract_name.utf8_chunks() {
                buf.push_str(u.valid());

                if !u.invalid().is_empty() {
                    buf.push('\u{FFFD}');
                }
            }
        } else if self.is_unnamed() {
            // nothing to do
        } else {
            // unreachable.
        }

        buf.fmt(f)
    }
}

impl FromStr for UnixAddr<'static> {
    type Err = ParseError;

    /// See [`UnixAddr::from_str`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UnixAddr::from_str(s).map(UnixAddr::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::UnixAddr;

    #[test]
    fn test_from_abstract_name_until_nul() {
        let _ = UnixAddr::from_abstract_name_until_nul::<true>(&[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        ])
        .unwrap();
        let _ = UnixAddr::from_abstract_name_until_nul::<true>(&[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        ])
        .unwrap_err();
    }

    #[test]
    fn test_from_abstract_name_bytes_until_nul() {
        let _ = UnixAddr::from_abstract_name_bytes_until_nul::<true>(&[
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        ])
        .unwrap();
        let _ = UnixAddr::from_abstract_name_bytes_until_nul::<true>(&[
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        ])
        .unwrap_err();
    }
}
