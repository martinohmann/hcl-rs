# Changelog

## [0.1.9](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.8...hcl-primitives-v0.1.9) - 2025-06-28

### Fixed

- restore clippy::pedantic and remove unused lifetime parameter

### Other

- Elide unused lifetime parameter

## [0.1.8](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.7...hcl-primitives-v0.1.8) - 2025-01-10

### Other

- *(deps)* bump itoa from 1.0.11 to 1.0.14 (#387)
- *(deps)* bump serde from 1.0.210 to 1.0.217 (#396)
- *(deps)* bump unicode-ident from 1.0.13 to 1.0.14 (#389)
- address clippy lints
- Fix all warnings (#391)

## [0.1.7](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.6...hcl-primitives-v0.1.7) - 2024-10-04

### Other

- *(deps)* bump unicode-ident from 1.0.12 to 1.0.13 ([#375](https://github.com/martinohmann/hcl-rs/pull/375))

## [0.1.6](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.5...hcl-primitives-v0.1.6) - 2024-09-25

### Other

- address linting issues
- *(deps)* bump ryu from 1.0.17 to 1.0.18 ([#358](https://github.com/martinohmann/hcl-rs/pull/358))
- *(deps)* bump kstring from 2.0.0 to 2.0.2 ([#364](https://github.com/martinohmann/hcl-rs/pull/364))
- *(deps)* bump serde from 1.0.197 to 1.0.210 ([#371](https://github.com/martinohmann/hcl-rs/pull/371))

## [0.1.5](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.4...hcl-primitives-v0.1.5) - 2024-05-16

### Other
- *(deps)* bump itoa from 1.0.9 to 1.0.11 ([#337](https://github.com/martinohmann/hcl-rs/pull/337))

## [0.1.4](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.3...hcl-primitives-v0.1.4) - 2024-04-13

### Other
- *(clippy)* address new pedantic lints in nightly

## [0.1.3](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.2...hcl-primitives-v0.1.3) - 2024-04-08

### Other
- *(deps)* bump ryu from 1.0.15 to 1.0.17 ([#325](https://github.com/martinohmann/hcl-rs/pull/325))

## [0.1.2](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.1...hcl-primitives-v0.1.2) (2024-01-04)


### Bug Fixes

* **cargo:** add more keywords ([7125a8c](https://github.com/martinohmann/hcl-rs/commit/7125a8cc05c95b9eaa872d8eb95840c583309575))

## [0.1.1](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.1.0...hcl-primitives-v0.1.1) (2023-06-15)


### Features

* **template:** add `{escape,unescape}_markers` ([#245](https://github.com/martinohmann/hcl-rs/issues/245)) ([5e51771](https://github.com/martinohmann/hcl-rs/commit/5e517713e9ba001306e2574fbbe1271bcfe1adda))

## [0.1.0](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.3...hcl-primitives-v0.1.0) (2023-05-02)


### ⚠ BREAKING CHANGES

* **ident:** `Ident::new` now returns `Ident` instead of `Result<Ident, Error>` and will panic if an invalid identifier is encountered. Use `Ident::try_new` instead to get the old behaviour.

### Features

* **ident:** add `Ident::try_new` ([#210](https://github.com/martinohmann/hcl-rs/issues/210)) ([4c15e1e](https://github.com/martinohmann/hcl-rs/commit/4c15e1e5b6eb7aedadef75da6a7fb11d5c9e8ec3))


### Bug Fixes

* **ident:** make `Ident::new` panic on invalid identifier ([#212](https://github.com/martinohmann/hcl-rs/issues/212)) ([bf8467a](https://github.com/martinohmann/hcl-rs/commit/bf8467ab759a78f43b9be3bc665bd29d46aa0baa))

## [0.0.3](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.2...hcl-primitives-v0.0.3) (2023-04-21)


### Features

* implement `From` for `Cow&lt;str&gt;` <-> `InternalString` ([e352a5a](https://github.com/martinohmann/hcl-rs/commit/e352a5ac0f0eb915b0d29cc44ec2c36f5d2d9c59))

## [0.0.2](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.1...hcl-primitives-v0.0.2) (2023-03-16)


### Features

* move `Strip`, `BinaryOperator` and `UnaryOperator` type defs ([20a8366](https://github.com/martinohmann/hcl-rs/commit/20a8366447e5f8673562cf37b9dda6bc8ffc6295))
