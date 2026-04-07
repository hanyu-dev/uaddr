# uaddr

[![test pipeline][test-pipeline-badge]][test-pipeline-url]
[![crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![license][license-badge]][license-url]

[test-pipeline-badge]: https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml/badge.svg
[test-pipeline-url]: https://github.com/hanyu-dev/uaddr/actions/workflows/ci.yml?query=branch%3Amain
[crates-badge]: https://img.shields.io/crates/v/uaddr
[crates-url]: https://crates.io/crates/uaddr
[docs-badge]: https://docs.rs/uaddr/badge.svg
[docs-url]: https://docs.rs/uaddr
[license-badge]: https://img.shields.io/crates/l/uaddr
[license-url]: https://opensource.org/licenses/MIT

A unified address type that can represent an IPv4 / IPv6 socket address, a UNIX
domain socket (UDS) address, or a hostname with a port. Supports `no_std`.

## Usage

Adds `uaddr` to your `Cargo.toml`:

```toml
[dependencies]
uaddr = "0.4"
```

Then parses any supported address format:

```rust
use std::net::SocketAddr;

use uaddr::UniAddr;

// IPv4 socket address
let addr = "127.0.0.1:8080".parse::<UniAddr>().unwrap();
assert!(matches!(addr, UniAddr::Inet(SocketAddr::V4(_))));

// IPv6 socket address
let addr = "[::1]:8080".parse::<UniAddr>().unwrap();
assert!(matches!(addr, UniAddr::Inet(SocketAddr::V6(_))));

// Hostname with port (resolved lazily)
let addr = "example.com:443".parse::<UniAddr>().unwrap();
assert!(matches!(addr, UniAddr::Host(_)));
assert!(addr.resolved().is_err());

// UNIX domain socket address - pathname address
let addr = "unix:/run/app.sock".parse::<UniAddr>().unwrap();
assert!(matches!(addr, UniAddr::Unix(_)));

// UNIX domain socket address - abstract address
let addr = "unix:@my-service".parse::<UniAddr>().unwrap();
assert!(matches!(addr, UniAddr::Unix(_)));
```

Please refer to the documentation for more details and examples.

## Changelog

[CHANGELOG.md](CHANGELOG.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
