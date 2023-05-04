# Changelog

## [0.5.0](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.8...hcl-edit-v0.5.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* **structure:** `Block::new` now only accepts a single `ident` argument. Set the block body, by updating `body` field of `Block`.
* **structure:** The `BlockBody` and `OnelineBody` types were removed. `Block` now directly uses `Body`. One-line blocks can still be constructed by calling `body.set_prefer_oneline(true)`.

### Code Refactoring

* **structure:** remove `BlockBody` and `OnelineBody` ([#218](https://github.com/martinohmann/hcl-rs/issues/218)) ([1267054](https://github.com/martinohmann/hcl-rs/commit/126705402ce7c95c10160cf64349e13f41b09f3f))
* **structure:** remove `body` argument from `Block::new` ([#220](https://github.com/martinohmann/hcl-rs/issues/220)) ([04c78f8](https://github.com/martinohmann/hcl-rs/commit/04c78f81d1e13561167872d3d6a2b4dd835f3d9f))

## [0.4.8](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.7...hcl-edit-v0.4.8) (2023-05-03)


### Features

* make constructors generic ([#216](https://github.com/martinohmann/hcl-rs/issues/216)) ([711fd74](https://github.com/martinohmann/hcl-rs/commit/711fd74b41c6b114f80d28d69e0d1a62654408fa))

## [0.4.7](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.6...hcl-edit-v0.4.7) (2023-05-02)


### Features

* **ident:** add `Ident::try_new` ([#210](https://github.com/martinohmann/hcl-rs/issues/210)) ([4c15e1e](https://github.com/martinohmann/hcl-rs/commit/4c15e1e5b6eb7aedadef75da6a7fb11d5c9e8ec3))
* **template:** add methods to ease `Element` access ([#214](https://github.com/martinohmann/hcl-rs/issues/214)) ([d4687b2](https://github.com/martinohmann/hcl-rs/commit/d4687b2bb6008040ae4f530de9dfd0c0efc4711f))
* **template:** implement `From` for `Template` and `StringTemplate` ([c569ccf](https://github.com/martinohmann/hcl-rs/commit/c569ccf39241b8e48c3554b36c8b91a46cc026c2))


### Bug Fixes

* **template:** dereference `StringTemplate` to `Template` ([#213](https://github.com/martinohmann/hcl-rs/issues/213)) ([906b3a0](https://github.com/martinohmann/hcl-rs/commit/906b3a0ef7ae9299ebda820e0570788688ea9814))


### Reverts

* **template:** dereference `StringTemplate` to `Template` ([#215](https://github.com/martinohmann/hcl-rs/issues/215)) ([bc20933](https://github.com/martinohmann/hcl-rs/commit/bc20933a6180cc1f832443d01af5fe758087c5bc))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * hcl-primitives bumped from 0.0.3 to 0.1.0

## [0.4.6](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.5...hcl-edit-v0.4.6) (2023-04-30)


### Features

* **structure:** add conversion methods to `BlockBody` ([e5001eb](https://github.com/martinohmann/hcl-rs/commit/e5001ebd9d98b804167df1650e6bc89a437bf6c0))
* **structure:** implement `IntoIterator` for `OnelineBody` ([cc69ba1](https://github.com/martinohmann/hcl-rs/commit/cc69ba1acb069fc4354b87ef335ce572b1d6ce20))

## [0.4.5](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.4...hcl-edit-v0.4.5) (2023-04-28)


### Performance Improvements

* **parser:** avoid `alt` in `array` and `object` parsers ([#205](https://github.com/martinohmann/hcl-rs/issues/205)) ([1c2ee01](https://github.com/martinohmann/hcl-rs/commit/1c2ee0185b9fb80bdd27d1735c1b53bbd168e6f6))

## [0.4.4](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.3...hcl-edit-v0.4.4) (2023-04-22)


### Features

* **expr:** add conversion methods to `Expression` ([679ad12](https://github.com/martinohmann/hcl-rs/commit/679ad124b978cfeb7eff88a41cdb9c22bc1d6c0f))
* **expr:** add conversion methods to `ObjectKey` ([e268c71](https://github.com/martinohmann/hcl-rs/commit/e268c71213e3d2ad3df4ebe8caa1e6b80d42d68d))

## [0.4.3](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.2...hcl-edit-v0.4.3) (2023-04-22)


### Features

* **expr:** add missing `From` impl for `Expression` ([bf235bc](https://github.com/martinohmann/hcl-rs/commit/bf235bc65b00bc0163fdadde82883294666f3b87))
* **structure:** add impl for `BlockLabel` ([22cc910](https://github.com/martinohmann/hcl-rs/commit/22cc9107459661b42a173076514ca2df4fda008f))
* **structure:** add missing `From` impls ([035b71e](https://github.com/martinohmann/hcl-rs/commit/035b71eb461c5a325b85633568175b812508e1a6))
* **template:** add missing `From` impls ([a558193](https://github.com/martinohmann/hcl-rs/commit/a558193ce2754c6d7a35239c2220bdb2ff73038c))

## [0.4.2](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.1...hcl-edit-v0.4.2) (2023-04-21)


### Features

* implement `From` for `Cow&lt;str&gt;` <-> `InternalString` ([e352a5a](https://github.com/martinohmann/hcl-rs/commit/e352a5ac0f0eb915b0d29cc44ec2c36f5d2d9c59))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * hcl-primitives bumped from 0.0.2 to 0.0.3

## [0.4.1](https://github.com/martinohmann/hcl-rs/compare/hcl-edit-v0.4.0...hcl-edit-v0.4.1) (2023-04-21)


### Features

* implement `Deref` for `RawString` ([fdce941](https://github.com/martinohmann/hcl-rs/commit/fdce941df3f58f3b8ecfc667bbf6f5c013bec191))
* implement `From` for `Cow&lt;str&gt;` <-> `RawString` ([de49aa4](https://github.com/martinohmann/hcl-rs/commit/de49aa4efbf3a5f985977a1cf905348683a8edee))


### Bug Fixes

* preserve body comments during encode ([316c16b](https://github.com/martinohmann/hcl-rs/commit/316c16b7e0b55cb007b4e26d84c509889650a564))

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
