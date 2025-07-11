# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.7](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.6...hcl2json-v0.1.7) - 2025-07-11

### Other

- updated the following local packages: hcl-rs

## [0.1.6](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.5...hcl2json-v0.1.6) - 2025-07-08

### Fixed

- sort dir entries for deterministic ordering

## [0.1.5](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.4...hcl2json-v0.1.5) - 2025-07-03

### Other

- updated the following local packages: hcl-rs

## [0.1.4](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.3...hcl2json-v0.1.4) - 2025-06-30

### Added

- better error context on processing errors

### Other

- improve CLI option documentation

## [0.1.3](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.2...hcl2json-v0.1.3) - 2025-06-30

### Fixed

- emit correct empty collection if paths are empty ([#446](https://github.com/martinohmann/hcl-rs/pull/446))

### Other

- remove `docs.rs` badge for `hcl2json`
- *(hcl2json)* add integration tests ([#448](https://github.com/martinohmann/hcl-rs/pull/448))

## [0.1.2](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.1...hcl2json-v0.1.2) - 2025-06-30

### Other

- *(hcl2json)* use `globset` for faster glob matching ([#445](https://github.com/martinohmann/hcl-rs/pull/445))
- address some clippy lints

## [0.1.1](https://github.com/martinohmann/hcl-rs/compare/hcl2json-v0.1.0...hcl2json-v0.1.1) - 2025-06-29

### Added

- *(hcl2json)* add `--file-paths` ([#443](https://github.com/martinohmann/hcl-rs/pull/443))
- *(hcl2json)* add `--continue-on-error` flag ([#442](https://github.com/martinohmann/hcl-rs/pull/442))

### Other

- *(hcl2json)* avoid temporary vec in `glob_files` ([#440](https://github.com/martinohmann/hcl-rs/pull/440))

## [0.1.0](https://github.com/martinohmann/hcl-rs/releases/tag/hcl2json-v0.1.0) - 2025-06-28

### Added

- add simple `hcl2json` CLI ([#438](https://github.com/martinohmann/hcl-rs/pull/438))
