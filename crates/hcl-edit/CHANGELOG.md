# Changelog

## [0.4.0](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.3.2...hcl-edit-v0.4.0) (2023-04-18)


### ⚠ BREAKING CHANGES

* `FuncArgs::new` does not take any arguments anymore to align with constructors of other collection types. Use

### Features

* add useful collection methods ([#197](https://github.com/martinohmann/hcl-rs/issues/197)) ([54f318d](https://github.com/martinohmann/hcl-rs/commit/54f318dfb793bf41272c9a1cc60148cfedcf3b23))

## [0.3.2](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.3.1...hcl-edit-v0.3.2) (2023-04-12)


### Features

* implement iterator traits for collections ([18ba590](https://github.com/martinohmann/hcl-rs/commit/18ba59001739c4a3b8b8781aac5118f53c03e101))

## [0.3.1](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.3.0...hcl-edit-v0.3.1) (2023-03-31)


### Bug Fixes

* **parser:** add missing `despan` call ([34897d6](https://github.com/martinohmann/hcl-rs/commit/34897d655f2e57eb97d3e7e0cdf3e0d68286ac15))
* **parser:** remove misplaced negation ([0af4f4e](https://github.com/martinohmann/hcl-rs/commit/0af4f4e878f920853ada1a7dd6d2714a08037681))

## [0.3.0](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.2.1...hcl-edit-v0.3.0) (2023-03-30)


### ⚠ BREAKING CHANGES

* **expr:** replace `&ObjectKey` with `ObjectKeyMut` in `ObjectIterMut` ([#191](https://github.com/martinohmann/hcl-rs/issues/191))

### Features

* add `visit` and `visit_mut` modules ([#187](https://github.com/martinohmann/hcl-rs/issues/187)) ([ec4914d](https://github.com/martinohmann/hcl-rs/commit/ec4914dfca4f05e5ff5d55c9897d06bea9de488e))


### Bug Fixes

* **expr:** replace `&ObjectKey` with `ObjectKeyMut` in `ObjectIterMut` ([#191](https://github.com/martinohmann/hcl-rs/issues/191)) ([c74c9f6](https://github.com/martinohmann/hcl-rs/commit/c74c9f69a501eac410571fee7e72ccbb7fb111aa))

## [0.2.1](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.2.0...hcl-edit-v0.2.1) (2023-03-30)


### Bug Fixes

* **parser:** handle `/` ambiguity in expression parser ([#189](https://github.com/martinohmann/hcl-rs/issues/189)) ([f4c3547](https://github.com/martinohmann/hcl-rs/commit/f4c35470f40871ae1060164ee8879a17c7a127cb)), closes [#188](https://github.com/martinohmann/hcl-rs/issues/188)

## [0.2.0](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.1.0...hcl-edit-v0.2.0) (2023-03-29)


### ⚠ BREAKING CHANGES

* rename `Oneline` to `OnelineBody`
* make fields without invariants public ([#185](https://github.com/martinohmann/hcl-rs/issues/185))

### Bug Fixes

* make fields without invariants public ([#185](https://github.com/martinohmann/hcl-rs/issues/185)) ([2f5c9ad](https://github.com/martinohmann/hcl-rs/commit/2f5c9ad0e3f62edd59ac434ffd7942f4f252edb8))
* rename `Oneline` to `OnelineBody` ([ed2f784](https://github.com/martinohmann/hcl-rs/commit/ed2f784c99dce981624d1c99465457b1a19a2da9))
