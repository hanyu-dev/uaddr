# uaddr

[![Test pipeline](https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml/badge.svg)](https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml?query=branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/uaddr)](https://crates.io/crates/uaddr)
[![Docs.rs](https://img.shields.io/docsrs/uaddr)](https://docs.rs/crate/uaddr/latest)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/license/mit/)

This crate provides a unified address type that can represent:

1. an IPv4 / IPv6 socket address;
1. a UNIX domain socket (UDS) address;
1. a host address.

The former name of this crate was `uni-addr` and we renamed it to `uaddr` since version 0.4.0.

## Migration from `uni-addr` (migration from 0.3.x to 0.4.0)

The key changes are:

1. `crate::unix::SocketAddr` is now removed, and we introduce `crate::unix::UnixAddr` instead.

   The former was a wrapper type over `std::os::unix::net::SocketAddr` and the latter is a standalone type that can represent both filesystem-based and abstract namespace UDS addresses, and can be converted to `std::os::unix::net::SocketAddr` if needed.

1. New dedicated `crate::host::HostAddr` type representing a host address.

   See below for the motivation.

1. `crate::UniAddrInner` is now removed, just do pattern matching against `crate::UniAddr` directly.

   Before 0.4.0, we stored `HostAddr` directly as a "host:port" style string, which was a design mistake. In order to prevent callers from directly constructing invalid `Host` variant content, we stored the variant as `UniAddrInner` and made `UniAddr` a wrapper type, which made pattern matching on `UniAddr` very cumbersome. We have now fixed this.

1. `no_std` support.

   Since `crate::unix::SocketAddr`, which wraps a `std::os::unix::net::SocketAddr`, has been deprecated, this library is now `no_std` compatible.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
