# Changelog

## 0.0.0 (2023-03-07)


### ⚠ BREAKING CHANGES

* **eval:** the `Evaluate` implementation of `TemplateExpr` returns a `Value` instead of a `String` now to support interpolation unwrapping.

### Bug Fixes

* **eval:** correctly handle interpolation unwrapping ([85bed59](https://github.com/martinohmann/hcl-rs/commit/85bed59d3a5b37542bd0daaa577e1c07cc12ac7a))
