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

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
