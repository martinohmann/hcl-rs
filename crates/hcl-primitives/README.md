# hcl-primitives

[![Build Status](https://github.com/martinohmann/hcl-rs/workflows/ci/badge.svg)](https://github.com/martinohmann/hcl-rs/actions?query=workflow%3Aci)
[![crates.io](https://img.shields.io/crates/v/hcl-primitives)](https://crates.io/crates/hcl-primitives)
[![docs.rs](https://img.shields.io/docsrs/hcl-primitives)](https://docs.rs/hcl-primitives)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Primitives used by the HCL sub-languages.

It is recommended to use [`hcl-rs`](https://docs.rs/hcl-rs) instead of
depending on `hcl-primitives` directly.

## Cargo features

The following features are available:

* `perf`: enables parser performance optimizations such as inlining of small
  strings on the stack. This feature is disabled by default. Enabling it will
  pull in `kstring` as a dependency. The `perf` feature depends on the `std`
  feature and enables it automatically.
* `serde`: Provides [`Serialize`](https://docs.rs/serde/latest/serde/ser/trait.Serialize.html)
  and [`Deserialize`](https://docs.rs/serde/latest/serde/de/trait.Deserialize.html)
  implementations for various types within this crate. This feature is disabled
  by default. Enabling it will pull in `serde` as a dependency.
* `std`: Use the Rust Standard Library as a dependency. Disabling this feature
  will allow usage in `#![no_std]` environments. This feature is enabled by
  default.

## Contributing

Contributions are welcome! Please read
[`CONTRIBUTING.md`](https://github.com/martinohmann/hcl-rs/blob/main/CONTRIBUTING.md)
before creating a PR.

## License

The source code of hcl-primitives is licensed under either of [Apache License, Version
2.0](https://github.com/martinohmann/hcl-rs/blob/main/LICENSE-APACHE) or [MIT
license](https://github.com/martinohmann/hcl-rs/blob/main/LICENSE-MIT) at your
option.
