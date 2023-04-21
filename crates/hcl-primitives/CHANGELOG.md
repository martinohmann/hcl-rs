# Changelog

## [0.1.0](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.3...hcl-primitives-v0.1.0) (2023-04-21)


### ⚠ BREAKING CHANGES

* **eval:** the `Evaluate` implementation of `TemplateExpr` returns a `Value` instead of a `String` now to support interpolation unwrapping.

### Features

* add `hcl-primitives` crate ([#178](https://github.com/martinohmann/hcl-rs/issues/178)) ([200a16f](https://github.com/martinohmann/hcl-rs/commit/200a16f8d0299b50e24b3e8808e17547eef8bb2b))
* implement `From` for `Cow&lt;str&gt;` <-> `InternalString` ([e352a5a](https://github.com/martinohmann/hcl-rs/commit/e352a5ac0f0eb915b0d29cc44ec2c36f5d2d9c59))
* move `Strip`, `BinaryOperator` and `UnaryOperator` type defs ([20a8366](https://github.com/martinohmann/hcl-rs/commit/20a8366447e5f8673562cf37b9dda6bc8ffc6295))


### Bug Fixes

* **eval:** correctly handle interpolation unwrapping ([85bed59](https://github.com/martinohmann/hcl-rs/commit/85bed59d3a5b37542bd0daaa577e1c07cc12ac7a))

## [0.0.3](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.2...hcl-primitives-v0.0.3) (2023-04-21)


### Features

* implement `From` for `Cow&lt;str&gt;` <-> `InternalString` ([e352a5a](https://github.com/martinohmann/hcl-rs/commit/e352a5ac0f0eb915b0d29cc44ec2c36f5d2d9c59))

## [0.0.2](https://github.com/martinohmann/hcl-rs/compare/hcl-primitives-v0.0.1...hcl-primitives-v0.0.2) (2023-03-16)


### Features

* move `Strip`, `BinaryOperator` and `UnaryOperator` type defs ([20a8366](https://github.com/martinohmann/hcl-rs/commit/20a8366447e5f8673562cf37b9dda6bc8ffc6295))
