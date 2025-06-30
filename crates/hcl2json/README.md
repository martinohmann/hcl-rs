# hcl2json

[![Build Status](https://github.com/martinohmann/hcl-rs/workflows/ci/badge.svg)](https://github.com/martinohmann/hcl-rs/actions?query=workflow%3Aci)
[![crates.io](https://img.shields.io/crates/v/hcl2json)](https://crates.io/crates/hcl2json)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

CLI program for converting HCL to JSON.

## Installation

```sh
cargo install hcl2json
```

## Usage

### Convert a file from `stdin`

```sh
cat file.hcl | hcl2json
```

### Convert multiple files to JSON

```sh
hcl2json file.hcl other-file.tf
```

**Note**: When converting multiple files or directories `hcl2json` emits a JSON
array. By passing `--file-paths/-P` the behaviour can be changed to produce a
JSON map keyed by input file path.

### Recursively convert files from a directory

```sh
hcl2json --glob '**/*.tf' dir/
```

**Note**: The command above is equivalent to `hcl2json dir/**/*.tf` but may
have slightly better performance when there are hundreds of matching files.

### Simplify and pretty-print

Simplify HCL expressions where possible and emit pretty-printed JSON:

```sh
hcl2json --simplify --pretty file.hcl
```

## Similar tools

- [tmccombs/hcl2json](https://github.com/tmccombs/hcl2json): Converts single
  HCL files. Sweet and simple.
- [Bonial-International-GmbH/hcl2json](https://github.com/Bonial-International-GmbH/hcl2json):
  Supports bulk conversion but is generally slower than this implementation.

## Contributing

Contributions are welcome! Please read
[`CONTRIBUTING.md`](https://github.com/martinohmann/hcl-rs/blob/main/CONTRIBUTING.md)
before creating a PR.

## License

The source code of hcl2json is licensed under either of [Apache License, Version
2.0](https://github.com/martinohmann/hcl-rs/blob/main/LICENSE-APACHE) or [MIT
license](https://github.com/martinohmann/hcl-rs/blob/main/LICENSE-MIT) at your
option.
