# uaddr

[![Test pipeline](https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml/badge.svg)](https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml?query=branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/uaddr)](https://crates.io/crates/uaddr)
[![Docs.rs](https://img.shields.io/docsrs/uaddr)](https://docs.rs/crate/uaddr/latest)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/license/mit/)

This crate provides a unified address type that can represent:

1. an IPv4 / IPv6 socket address;
1. a UNIX domain socket (UDS) address;
1. a hostname with a port.

## Migration from `uni-addr`

`uni-addr` was renamed to `uaddr` since version 0.4.0. The key changes between 0.3.x and 0.4.0 are as follows.

1. Added: type `UnixAddr` representing a UNIX domain socket address.

   See below for the motivation.

1. Breaking change: type `SocketAddr` for UNIX platform is now removed.

   Type `SocketAddr` for UNIX platform was a wrapper over `SocketAddr` provided by the standard library, which is quite bloated (~128 bytes), and there're also limitations in terms of the APIs. We then replace it with `UnixAddr` which can be converted to `SocketAddr` lazily if needed.

1. Added: type `HostAddr` representing a hostname with a port (`hostname:port`) was introduced.

   See below for the motivation.

1. Breaking change: type `UniAddrInner` is now removed.

   Before 0.4.0, the `Host` variant of the address type enum contained an `Arc<str>` to store the validated "hostname:port". To prevent callers from bypassing validation and directly constructing the `Host` variant, the enum type was named `UniAddrInner` and wrapped with `UniAddr`. After 0.4.0, we fixed this flawed design.

1. Added: `no_std` support.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
