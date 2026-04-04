# CHANGELOG

## v0.4.0

- **Breaking**

  `uni-addr` was renamed to `uaddr`.

- **Added**

  `UnixAddr`, a dedicated type for UNIX domain socket addresses.

- **Breaking**

  `SocketAddr` (UNIX-only wrapper) is removed and replaced by `UnixAddr`. The old wrapper was ~128 bytes and had limited APIs; `UnixAddr` can be converted to `std::os::unix::net::SocketAddr` lazily when needed.

- **Added**

   `HostAddr`, a dedicated type for validated `hostname:port` pairs.

- **Breaking**

  `UniAddrInner` is now removed. Previously, the `Host` variant of `UniAddr` held an `Arc<str>` and the outer enum was private (`UniAddrInner`) to prevent bypassing validation. This design is now fixed.

- **Added**

  `no_std` support.
