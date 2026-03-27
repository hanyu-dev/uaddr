//! Glue codes

#[cfg(all(unix, feature = "std"))]
mod std {
    use std::ffi::OsStr;
    use std::io;
    #[cfg(target_os = "android")]
    use std::os::android::net::SocketAddrExt as _;
    #[cfg(target_os = "linux")]
    use std::os::linux::net::SocketAddrExt as _;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::net::SocketAddr;
    use std::path::Path;

    use crate::error::{InvalidUniAddr, ParseError};
    use crate::unix::UnixAddr;
    use crate::UniAddr;

    impl TryFrom<UnixAddr<'_>> for SocketAddr {
        type Error = io::Error;

        fn try_from(value: UnixAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&UnixAddr<'_>> for SocketAddr {
        type Error = io::Error;

        fn try_from(value: &UnixAddr<'_>) -> Result<Self, Self::Error> {
            if let Some(pathname) = value.as_pathname() {
                return Self::from_pathname(Path::new(OsStr::from_bytes(pathname)));
            }

            #[cfg(any(target_os = "linux", target_os = "android"))]
            if let Some(abstract_name) = value.as_abstract_name() {
                return Self::from_abstract_name(abstract_name);
            }

            if value.is_unnamed() {
                return Self::from_pathname(Path::new(OsStr::from_bytes(&[])));
            }

            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                InvalidUniAddr::Unsupported,
            ))
        }
    }

    impl TryFrom<SocketAddr> for UnixAddr<'static> {
        type Error = ParseError;

        fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
            UnixAddr::try_from(&value).map(UnixAddr::to_owned)
        }
    }

    impl<'a> TryFrom<&'a SocketAddr> for UnixAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a SocketAddr) -> Result<Self, Self::Error> {
            // ! Should sync with the implementation of
            // ! `TryFrom<&tokio::net::unix::SocketAddr>` for `UnixAddr` in
            // ! `tokio` module.

            if let Some(pathname) = value.as_pathname() {
                return UnixAddr::from_pathname(pathname.as_os_str().as_bytes());
            }

            #[cfg(any(target_os = "linux", target_os = "android"))]
            if let Some(abstract_name) = value.as_abstract_name() {
                return UnixAddr::from_abstract_name::<true>(abstract_name);
            }

            if value.is_unnamed() {
                return Ok(UnixAddr::new_unnamed());
            }

            Err(ParseError::InvalidUnixAddr)
        }
    }

    impl TryFrom<SocketAddr> for UniAddr<'static> {
        type Error = ParseError;

        fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
            UniAddr::try_from(&value).map(UniAddr::to_owned)
        }
    }

    impl<'a> TryFrom<&'a SocketAddr> for UniAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a SocketAddr) -> Result<Self, Self::Error> {
            UnixAddr::try_from(value).map(UniAddr::Unix)
        }
    }

    impl TryFrom<UniAddr<'_>> for SocketAddr {
        type Error = io::Error;

        fn try_from(value: UniAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&UniAddr<'_>> for SocketAddr {
        type Error = io::Error;

        fn try_from(value: &UniAddr<'_>) -> Result<Self, Self::Error> {
            match value {
                UniAddr::Unix(addr) => Self::try_from(addr),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    InvalidUniAddr::Unsupported,
                )),
            }
        }
    }
}

#[cfg(feature = "socket2")]
mod socket2 {
    use std::io;

    use socket2::SockAddr;

    use crate::error::{InvalidUniAddr, ParseError};
    use crate::host::HostAddr;
    use crate::unix::UnixAddr;
    use crate::UniAddr;

    impl TryFrom<UnixAddr<'_>> for socket2::SockAddr {
        type Error = io::Error;

        fn try_from(value: UnixAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    #[cfg(unix)]
    impl TryFrom<&UnixAddr<'_>> for socket2::SockAddr {
        type Error = io::Error;

        fn try_from(value: &UnixAddr<'_>) -> Result<Self, Self::Error> {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;
            use std::path::Path;

            Self::unix(Path::new(OsStr::from_bytes(value.as_ref())))
        }
    }

    #[cfg(not(unix))]
    impl TryFrom<&UnixAddr<'_>> for socket2::SockAddr {
        type Error = io::Error;

        fn try_from(value: &UnixAddr<'_>) -> Result<Self, Self::Error> {
            Err(io::Error::other(InvalidUniAddr::Unsupported))
        }
    }

    impl TryFrom<HostAddr<'_>> for socket2::SockAddr {
        type Error = io::Error;

        fn try_from(value: HostAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&HostAddr<'_>> for socket2::SockAddr {
        type Error = io::Error;

        fn try_from(value: &HostAddr<'_>) -> Result<Self, Self::Error> {
            value.resolved().map_or_else(
                || Err(io::Error::other(InvalidUniAddr::Unresolved)),
                |addr| Ok(Self::from(addr)),
            )
        }
    }

    impl TryFrom<UniAddr<'_>> for SockAddr {
        type Error = io::Error;

        fn try_from(value: UniAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&UniAddr<'_>> for SockAddr {
        type Error = io::Error;

        fn try_from(value: &UniAddr<'_>) -> Result<Self, Self::Error> {
            match value {
                UniAddr::Inet(addr) => Ok(Self::from(*addr)),
                UniAddr::Unix(addr) => Self::try_from(addr),
                UniAddr::Host(addr) => Self::try_from(addr),
            }
        }
    }

    impl TryFrom<SockAddr> for UnixAddr<'static> {
        type Error = ParseError;

        fn try_from(value: SockAddr) -> Result<Self, Self::Error> {
            UnixAddr::try_from(&value).map(UnixAddr::to_owned)
        }
    }

    #[cfg(unix)]
    impl<'a> TryFrom<&'a socket2::SockAddr> for UnixAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a socket2::SockAddr) -> Result<Self, Self::Error> {
            use std::os::unix::ffi::OsStrExt;

            if let Some(pathname) = value.as_pathname() {
                return UnixAddr::from_pathname(pathname.as_os_str().as_bytes());
            }

            #[cfg(any(target_os = "linux", target_os = "android"))]
            if let Some(abstract_name) = value.as_abstract_namespace() {
                return UnixAddr::from_abstract_name::<true>(abstract_name);
            }

            if value.is_unnamed() {
                return Ok(UnixAddr::new_unnamed());
            }

            Err(ParseError::InvalidUnixAddr)
        }
    }

    #[cfg(not(unix))]
    impl<'a> TryFrom<&'a socket2::SockAddr> for UnixAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a socket2::SockAddr) -> Result<Self, Self::Error> {
            Err(ParseError::Unsupported)
        }
    }

    impl TryFrom<SockAddr> for UniAddr<'static> {
        type Error = ParseError;

        fn try_from(value: SockAddr) -> Result<Self, Self::Error> {
            UniAddr::try_from(&value).map(UniAddr::to_owned)
        }
    }

    impl<'a> TryFrom<&'a SockAddr> for UniAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a SockAddr) -> Result<Self, Self::Error> {
            if let Some(addr) = value.as_socket() {
                return Ok(UniAddr::Inet(addr));
            }

            UnixAddr::try_from(value).map(UniAddr::Unix)
        }
    }
}

#[cfg(all(unix, feature = "tokio"))]
mod tokio {
    use std::io;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::net::SocketAddr;

    use crate::error::{InvalidUniAddr, ParseError};
    use crate::unix::UnixAddr;
    use crate::UniAddr;

    impl TryFrom<UnixAddr<'_>> for tokio::net::unix::SocketAddr {
        type Error = io::Error;

        fn try_from(value: UnixAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&UnixAddr<'_>> for tokio::net::unix::SocketAddr {
        type Error = io::Error;

        fn try_from(value: &UnixAddr<'_>) -> Result<Self, Self::Error> {
            SocketAddr::try_from(value).map(Self::from)
        }
    }

    impl TryFrom<tokio::net::unix::SocketAddr> for UnixAddr<'static> {
        type Error = ParseError;

        fn try_from(value: tokio::net::unix::SocketAddr) -> Result<Self, Self::Error> {
            UnixAddr::try_from(&value).map(UnixAddr::to_owned)
        }
    }

    impl<'a> TryFrom<&'a tokio::net::unix::SocketAddr> for UnixAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a tokio::net::unix::SocketAddr) -> Result<Self, Self::Error> {
            // ! Should sync with the implementation of `TryFrom<&SocketAddr>` for
            // ! `UnixAddr` in `std` module.

            if let Some(pathname) = value.as_pathname() {
                return UnixAddr::from_pathname(pathname.as_os_str().as_bytes());
            }

            #[cfg(any(target_os = "linux", target_os = "android"))]
            if let Some(abstract_name) = value.as_abstract_name() {
                return UnixAddr::from_abstract_name::<true>(abstract_name);
            }

            if value.is_unnamed() {
                return Ok(UnixAddr::new_unnamed());
            }

            Err(ParseError::InvalidUnixAddr)
        }
    }

    impl TryFrom<tokio::net::unix::SocketAddr> for UniAddr<'static> {
        type Error = ParseError;

        fn try_from(value: tokio::net::unix::SocketAddr) -> Result<Self, Self::Error> {
            UniAddr::try_from(&value).map(UniAddr::to_owned)
        }
    }

    impl<'a> TryFrom<&'a tokio::net::unix::SocketAddr> for UniAddr<'a> {
        type Error = ParseError;

        fn try_from(value: &'a tokio::net::unix::SocketAddr) -> Result<Self, Self::Error> {
            UnixAddr::try_from(value).map(UniAddr::Unix)
        }
    }

    impl TryFrom<UniAddr<'_>> for tokio::net::unix::SocketAddr {
        type Error = io::Error;

        fn try_from(value: UniAddr<'_>) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&UniAddr<'_>> for tokio::net::unix::SocketAddr {
        type Error = io::Error;

        fn try_from(value: &UniAddr<'_>) -> Result<Self, Self::Error> {
            match value {
                UniAddr::Unix(addr) => Self::try_from(addr),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    InvalidUniAddr::Unsupported,
                )),
            }
        }
    }

    #[cfg(test)]
    #[test]
    fn test_try_from() {}
}

#[cfg(feature = "serde")]
mod serde {
    use alloc::string::ToString;

    use crate::host::HostAddr;
    use crate::unix::UnixAddr;
    use crate::UniAddr;

    macro_rules! impl_serde {
        ($ty:ident) => {
            impl ::serde::Serialize for $ty<'_> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::serde::Serializer,
                {
                    self.to_string().serialize(serializer)
                }
            }

            impl<'de> ::serde::Deserialize<'de> for $ty<'de> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    struct Visitor;

                    impl<'de> ::serde::de::Visitor<'de> for Visitor {
                        type Value = $ty<'de>;

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            <$ty>::from_str(v).map(<$ty>::to_owned).map_err(E::custom)
                        }

                        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            <$ty>::from_str(v).map_err(E::custom)
                        }

                        fn expecting(
                            &self,
                            formatter: &mut ::core::fmt::Formatter,
                        ) -> ::core::fmt::Result {
                            formatter.write_str(concat!(
                                "a string representation of a ",
                                stringify!($ty)
                            ))
                        }
                    }

                    deserializer.deserialize_str(Visitor)
                }
            }
        };
    }

    impl_serde!(UnixAddr);
    impl_serde!(HostAddr);
    impl_serde!(UniAddr);

    #[cfg(test)]
    #[test]
    fn assert_tokens() {
        use serde_test::{assert_tokens, Token};

        let addr = UnixAddr::from_str("unix:/path/to/your/file.socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:/path/to/your/file.socket")]);

        let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:@abstract-socket")]);

        let addr = UnixAddr::from_str("unix:").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:")]);

        let addr = UnixAddr::from_bytes(b"/path/to/your/file.socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:/path/to/your/file.socket")]);

        let addr = UnixAddr::from_bytes(b"\0abstract-socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:@abstract-socket")]);

        let addr = UnixAddr::from_bytes(b"").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:")]);

        let addr = HostAddr::from_str("example.com:8080").unwrap();
        assert_tokens(&addr, &[Token::Str("example.com:8080")]);

        let addr = UniAddr::from_str("127.0.0.1:13168").unwrap();
        assert_tokens(&addr, &[Token::Str("127.0.0.1:13168")]);

        let addr = UniAddr::from_str("[::1]:13168").unwrap();
        assert_tokens(&addr, &[Token::Str("[::1]:13168")]);

        let addr = UniAddr::from_str("unix:/path/to/your/file.socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:/path/to/your/file.socket")]);

        let addr = UniAddr::from_str("unix:@abstract-socket").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:@abstract-socket")]);

        let addr = UniAddr::from_str("unix:").unwrap();
        assert_tokens(&addr, &[Token::Str("unix:")]);

        let addr = UniAddr::from_str("example.com:8080").unwrap();
        assert_tokens(&addr, &[Token::Str("example.com:8080")]);
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    #[cfg(unix)]
    use crate::unix::UnixAddr;
    use crate::UniAddr;

    #[cfg(unix)]
    fn test_unix_addr_try_from_general<T>(
        as_pathname: impl Fn(&T) -> Option<&[u8]>,
        #[cfg(any(target_os = "linux", target_os = "android"))] as_abstract_name: impl Fn(
            &T,
        )
            -> Option<
            &[u8],
        >,
        as_unnamed: impl Fn(&T) -> Option<&[u8]>,
    ) where
        T: for<'a> TryFrom<UnixAddr<'a>, Error: core::fmt::Debug>,
        T: for<'a, 'b> TryFrom<&'a UnixAddr<'b>, Error: core::fmt::Debug>,
        UnixAddr<'static>: TryFrom<T, Error: core::fmt::Debug>,
        for<'a> UnixAddr<'a>: TryFrom<&'a T, Error: core::fmt::Debug>,
    {
        macro_rules! test {
            ($bytes:expr, $fn:ident) => {
                let uaddr = UnixAddr::from_bytes($bytes).unwrap();

                let addr = T::try_from(&uaddr).unwrap();
                assert_eq!($fn(&addr), Some($bytes));
                assert_eq!(UnixAddr::try_from(&addr).unwrap(), uaddr);
                assert_eq!(UnixAddr::try_from(addr).unwrap(), uaddr);

                let addr = T::try_from(uaddr.clone()).unwrap();
                assert_eq!($fn(&addr), Some($bytes));
                assert_eq!(UnixAddr::try_from(&addr).unwrap(), uaddr);
                assert_eq!(UnixAddr::try_from(addr).unwrap(), uaddr);
            };
            ($bytes:expr, $expected:expr, $fn:ident) => {
                let uaddr = UnixAddr::from_bytes($bytes).unwrap();

                let addr = T::try_from(&uaddr).unwrap();
                assert_eq!($fn(&addr), Some($expected));
                assert_eq!(UnixAddr::try_from(&addr).unwrap(), uaddr);
                assert_eq!(UnixAddr::try_from(addr).unwrap(), uaddr);

                let addr = T::try_from(uaddr.clone()).unwrap();
                assert_eq!($fn(&addr), Some($expected));
                assert_eq!(UnixAddr::try_from(&addr).unwrap(), uaddr);
                assert_eq!(UnixAddr::try_from(addr).unwrap(), uaddr);
            };
        }

        test!(&b"/path/to/your/file.socket"[..], as_pathname);

        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            test!(
                &b"\0abstract-socket"[..],
                &b"abstract-socket"[..],
                as_abstract_name
            );
        }

        test!(&b""[..], as_unnamed);
    }

    #[test]
    fn test_uni_addr_try_from_std() {
        let addr = "127.0.0.1:13168".parse::<SocketAddr>().unwrap();
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(SocketAddr::try_from(&uaddr).unwrap(), addr);
        assert_eq!(SocketAddr::try_from(uaddr).unwrap(), addr);

        let addr = "[::1]:13168".parse::<SocketAddr>().unwrap();
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(SocketAddr::try_from(&uaddr).unwrap(), addr);
        assert_eq!(SocketAddr::try_from(uaddr).unwrap(), addr);

        let mut addr = "localhost:8080".parse::<UniAddr>().unwrap();
        let _ = SocketAddr::try_from(&addr).unwrap_err();
        if addr.blocking_resolve_host_name().is_ok() {
            let addr = SocketAddr::try_from(&addr).unwrap();
            assert!(addr.ip().is_loopback());
        }

        #[cfg(unix)]
        {
            use std::os::unix::net::SocketAddr;

            let addr = UnixAddr::from_str("unix:/path/to/your/file.socket").unwrap();
            let addr = SocketAddr::try_from(&addr).unwrap();
            let uaddr = UniAddr::try_from(&addr).unwrap();
            assert_eq!(
                SocketAddr::try_from(&uaddr).unwrap().as_pathname(),
                addr.as_pathname()
            );
            assert_eq!(
                SocketAddr::try_from(uaddr).unwrap().as_pathname(),
                addr.as_pathname()
            );

            #[cfg(any(target_os = "linux", target_os = "android"))]
            {
                #[cfg(target_os = "android")]
                use std::os::android::net::SocketAddrExt as _;
                #[cfg(target_os = "linux")]
                use std::os::linux::net::SocketAddrExt as _;

                let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
                let addr = SocketAddr::try_from(&addr).unwrap();
                let uaddr = UniAddr::try_from(&addr).unwrap();
                assert_eq!(
                    SocketAddr::try_from(&uaddr).unwrap().as_abstract_name(),
                    addr.as_abstract_name()
                );
                assert_eq!(
                    SocketAddr::try_from(uaddr).unwrap().as_abstract_name(),
                    addr.as_abstract_name()
                );
            }

            let addr = UnixAddr::from_str("unix:").unwrap();
            let addr = SocketAddr::try_from(&addr).unwrap();
            let uaddr = UniAddr::try_from(&addr).unwrap();
            assert_eq!(
                SocketAddr::try_from(&uaddr).unwrap().is_unnamed(),
                addr.is_unnamed()
            );
            assert_eq!(
                SocketAddr::try_from(uaddr).unwrap().is_unnamed(),
                addr.is_unnamed()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_uni_addr_try_from_tokio() {
        use tokio::net::unix::SocketAddr;

        let addr = UnixAddr::from_str("unix:/path/to/your/file.socket").unwrap();
        let addr = SocketAddr::try_from(&addr).unwrap();
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(
            SocketAddr::try_from(&uaddr).unwrap().as_pathname(),
            addr.as_pathname()
        );
        assert_eq!(
            SocketAddr::try_from(uaddr).unwrap().as_pathname(),
            addr.as_pathname()
        );

        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
            let addr = SocketAddr::try_from(&addr).unwrap();
            let uaddr = UniAddr::try_from(&addr).unwrap();
            assert_eq!(
                SocketAddr::try_from(&uaddr).unwrap().as_abstract_name(),
                addr.as_abstract_name()
            );
            assert_eq!(
                SocketAddr::try_from(uaddr).unwrap().as_abstract_name(),
                addr.as_abstract_name()
            );
        }

        let addr = UnixAddr::from_str("unix:").unwrap();
        let addr = SocketAddr::try_from(&addr).unwrap();
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(
            SocketAddr::try_from(&uaddr).unwrap().is_unnamed(),
            addr.is_unnamed()
        );
        assert_eq!(
            SocketAddr::try_from(uaddr).unwrap().is_unnamed(),
            addr.is_unnamed()
        );
    }

    #[test]
    fn test_uni_addr_try_from_socket2() {
        use socket2::SockAddr;

        let addr = "127.0.0.1:13168".parse::<SocketAddr>().unwrap();
        let addr = SockAddr::from(addr);
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(SockAddr::try_from(&uaddr).unwrap(), addr);
        assert_eq!(SockAddr::try_from(uaddr).unwrap(), addr);

        let addr = "[::1]:13168".parse::<SocketAddr>().unwrap();
        let addr = SockAddr::from(addr);
        let uaddr = UniAddr::try_from(&addr).unwrap();
        assert_eq!(SockAddr::try_from(&uaddr).unwrap(), addr);
        assert_eq!(SockAddr::try_from(uaddr).unwrap(), addr);

        let mut addr = "localhost:8080".parse::<UniAddr>().unwrap();
        let _ = SockAddr::try_from(&addr).unwrap_err();
        if addr.blocking_resolve_host_name().is_ok() {
            let addr = SockAddr::try_from(&addr).unwrap();
            let addr = addr.as_socket().unwrap();
            assert!(addr.ip().is_loopback());
        }

        #[cfg(unix)]
        {
            let addr = UnixAddr::from_str("unix:/path/to/your/file.socket").unwrap();
            let addr = SockAddr::try_from(&addr).unwrap();
            let uaddr = UniAddr::try_from(&addr).unwrap();
            assert_eq!(
                SockAddr::try_from(&uaddr).unwrap().as_pathname(),
                addr.as_pathname()
            );
            assert_eq!(
                SockAddr::try_from(uaddr).unwrap().as_pathname(),
                addr.as_pathname()
            );

            #[cfg(any(target_os = "linux", target_os = "android"))]
            {
                let addr = UnixAddr::from_str("unix:@abstract-socket").unwrap();
                let addr = SockAddr::try_from(&addr).unwrap();
                let uaddr = UniAddr::try_from(&addr).unwrap();
                assert_eq!(
                    SockAddr::try_from(&uaddr).unwrap().as_abstract_namespace(),
                    addr.as_abstract_namespace()
                );
                assert_eq!(
                    SockAddr::try_from(uaddr).unwrap().as_abstract_namespace(),
                    addr.as_abstract_namespace()
                );
            }

            let addr = UnixAddr::from_str("unix:").unwrap();
            let addr = SockAddr::try_from(&addr).unwrap();
            let uaddr = UniAddr::try_from(&addr).unwrap();
            assert_eq!(
                SockAddr::try_from(&uaddr).unwrap().is_unnamed(),
                addr.is_unnamed()
            );
            assert_eq!(
                SockAddr::try_from(uaddr).unwrap().is_unnamed(),
                addr.is_unnamed()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_addr_try_from_std() {
        #[cfg(target_os = "android")]
        use std::os::android::net::SocketAddrExt as _;
        #[cfg(target_os = "linux")]
        use std::os::linux::net::SocketAddrExt as _;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::net::SocketAddr;

        test_unix_addr_try_from_general::<SocketAddr>(
            |addr| addr.as_pathname().map(|v| v.as_os_str().as_bytes()),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            |addr| addr.as_abstract_name(),
            |addr| addr.is_unnamed().then_some(&[]),
        );
    }

    #[cfg(all(unix, feature = "tokio"))]
    #[test]
    fn test_unix_addr_try_from_tokio() {
        use std::os::unix::ffi::OsStrExt;

        test_unix_addr_try_from_general::<tokio::net::unix::SocketAddr>(
            |addr| addr.as_pathname().map(|v| v.as_os_str().as_bytes()),
            |addr| addr.as_abstract_name(),
            |addr| addr.is_unnamed().then_some(&[]),
        );
    }

    #[cfg(all(unix, feature = "socket2"))]
    #[test]
    fn test_unix_addr_try_from_socket2() {
        use std::os::unix::ffi::OsStrExt;

        test_unix_addr_try_from_general::<socket2::SockAddr>(
            |addr| addr.as_pathname().map(|v| v.as_os_str().as_bytes()),
            |addr| addr.as_abstract_namespace(),
            |addr| addr.is_unnamed().then_some(&[]),
        );
    }
}
