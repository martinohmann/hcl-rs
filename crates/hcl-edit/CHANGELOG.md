# Changelog

## [0.2.0](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.1.0...hcl-edit-v0.2.0) (2023-03-24)


### ⚠ BREAKING CHANGES

* **eval:** the `Evaluate` implementation of `TemplateExpr` returns a `Value` instead of a `String` now to support interpolation unwrapping.

### Features

* add `hcl-edit` crate ([#182](https://github.com/martinohmann/hcl-rs/issues/182)) ([b29bdb5](https://github.com/martinohmann/hcl-rs/commit/b29bdb540ad4271a2b6bbc121614b1045a5081f2))


### Bug Fixes

* **eval:** correctly handle interpolation unwrapping ([85bed59](https://github.com/martinohmann/hcl-rs/commit/85bed59d3a5b37542bd0daaa577e1c07cc12ac7a))
