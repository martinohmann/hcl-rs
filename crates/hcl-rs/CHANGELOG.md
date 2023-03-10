# Changelog

## [0.13.3](https://github.com/martinohmann/hcl-rs/compare/hcl-rs-v0.13.2...hcl-rs-v0.13.3) (2023-03-10)


### Features

* add `hcl-primitives` crate ([#178](https://github.com/martinohmann/hcl-rs/issues/178)) ([200a16f](https://github.com/martinohmann/hcl-rs/commit/200a16f8d0299b50e24b3e8808e17547eef8bb2b))

## [0.13.2](https://github.com/martinohmann/hcl-rs/compare/hcl-rs-v0.13.1...hcl-rs-v0.13.2) (2023-03-10)


### Bug Fixes

* **format:** do not unescape heredoc strings ([#171](https://github.com/martinohmann/hcl-rs/issues/171)) ([c2b37ec](https://github.com/martinohmann/hcl-rs/commit/c2b37ec29539bc51e3503f902e0e46ad57e2264c))

## [0.13.1](https://github.com/martinohmann/hcl-rs/compare/v0.13.0...v0.13.1) (2023-03-07)


### Miscellaneous

* dual-license under MIT and Apache 2.0 ([#165](https://github.com/martinohmann/hcl-rs/issues/165)) ([48fe290](https://github.com/martinohmann/hcl-rs/commit/48fe2908a87c07713d64975a9eb63b8258d6d1a4))

## [0.13.0](https://github.com/martinohmann/hcl-rs/compare/v0.12.3...v0.13.0) (2023-03-01)


### ⚠ BREAKING CHANGES

* **perf:** The trait bounds on `Identifier::{new,unchecked}` and `Variable::{new,unchecked}` were changed from `Into<String>` to `Into<InternalString>`. This has no impact on common use cases involving std types but may break code using custom types.

### Features

* **perf:** optionally inline identifier strings on the stack ([#163](https://github.com/martinohmann/hcl-rs/issues/163)) ([d810d55](https://github.com/martinohmann/hcl-rs/commit/d810d556ab5ecea7a725b7beb602053a531c55e2))


### Miscellaneous

* **deps:** bump actions/cache from 3.2.2 to 3.2.4 ([#160](https://github.com/martinohmann/hcl-rs/issues/160)) ([790b385](https://github.com/martinohmann/hcl-rs/commit/790b3856c24913b9e6b92e4c1efe70ea729bdc3e))
* **deps:** bump actions/cache from 3.2.4 to 3.2.6 ([#164](https://github.com/martinohmann/hcl-rs/issues/164)) ([5609b85](https://github.com/martinohmann/hcl-rs/commit/5609b8586e4ea2a9493a15ac9663ff4391790cb6))
* **deps:** update indoc requirement from 1.0 to 2.0 ([#161](https://github.com/martinohmann/hcl-rs/issues/161)) ([3ed52a8](https://github.com/martinohmann/hcl-rs/commit/3ed52a82264d753b6b4c75c7c044bb6d8e0e0cd8))

## [0.12.3](https://github.com/martinohmann/hcl-rs/compare/v0.12.2...v0.12.3) (2023-01-24)


### Bug Fixes

* do not strip escaped whitespace from strings ([24d0e6a](https://github.com/martinohmann/hcl-rs/commit/24d0e6a40b040a00819a70ad5ffb727820b95ee2))
* **parser:** automatically dedent indented heredocs ([#157](https://github.com/martinohmann/hcl-rs/issues/157)) ([011e95e](https://github.com/martinohmann/hcl-rs/commit/011e95e85f0681d11c12f4c0b01791d1e847e511))

## [0.12.2](https://github.com/martinohmann/hcl-rs/compare/v0.12.1...v0.12.2) (2023-01-20)


### Features

* **eval:** add `ErrorKind::RawExpression` ([1e89130](https://github.com/martinohmann/hcl-rs/commit/1e891309c25cdbd57cc17f3c71ec162f3bf685c0))


### Bug Fixes

* **benches:** add back deleted benchmark fixture ([d209687](https://github.com/martinohmann/hcl-rs/commit/d209687980bd4c5f058e03fede02615f0241c867))
* **de:** correct too restrictive trait bound ([42cff38](https://github.com/martinohmann/hcl-rs/commit/42cff38e9dad70c5c7d7305720338c8a6394570c))
* **parser:** preserve whitespace in adjacent template literal ([0f5a639](https://github.com/martinohmann/hcl-rs/commit/0f5a63935c3666d11f392a5cbab2b895c0fc9032))

## [0.12.1](https://github.com/martinohmann/hcl-rs/compare/v0.12.0...v0.12.1) (2023-01-12)


### Bug Fixes

* **eval:** implement correct template literal stripping ([#153](https://github.com/martinohmann/hcl-rs/issues/153)) ([d13db03](https://github.com/martinohmann/hcl-rs/commit/d13db031772c5597e28bb53baf8dc0f124b92159))

## [0.12.0](https://github.com/martinohmann/hcl-rs/compare/v0.11.2...v0.12.0) (2023-01-06)


### ⚠ BREAKING CHANGES

* **structure:** The signature of `Block::new` was changed from `Block::new(ident, labels, body)` to `Block::new(ident)`. If you depend on the previos behaviour, use `Body::from((ident, labels, body))` instead.

### Features

* **format:** allow formatting to `String` and `Vec&lt;u8&gt;` directly ([#151](https://github.com/martinohmann/hcl-rs/issues/151)) ([2384e39](https://github.com/martinohmann/hcl-rs/commit/2384e3972db89dc3301a22857ae4dc6a0025815f))
* **structure:** implement all structure iterators for `Body` ([#149](https://github.com/martinohmann/hcl-rs/issues/149)) ([ee752fa](https://github.com/martinohmann/hcl-rs/commit/ee752fad3d9bee3fab73656db5004cb4aaf47ee1))


### Bug Fixes

* **expr:** remove unused generics in `TraversalBuilder` ([56cbeb5](https://github.com/martinohmann/hcl-rs/commit/56cbeb557af86aa6ad0d8d2808cd5d0ab093b3d8))
* **format:** remove unnecessary generic from `FormatterBuilder` ([010932d](https://github.com/martinohmann/hcl-rs/commit/010932d063d2cc2637fcec49633fd65b771b8612))
* **structure:** update signature of `Block::new` ([2622f4d](https://github.com/martinohmann/hcl-rs/commit/2622f4deb156d95bc3fc9f4513861b094db8d32a))


### Miscellaneous

* **deps:** bump actions/cache from 3.0.11 to 3.2.2 ([#148](https://github.com/martinohmann/hcl-rs/issues/148)) ([10c550a](https://github.com/martinohmann/hcl-rs/commit/10c550a0a8100a98ed1afde964f9d0a1601fae12))
* **license:** update year ([faee584](https://github.com/martinohmann/hcl-rs/commit/faee5844d83d324caefb6993e6bfcbb468f23910))
* **test:** convert most of the tests into integration tests ([#152](https://github.com/martinohmann/hcl-rs/issues/152)) ([3361d9d](https://github.com/martinohmann/hcl-rs/commit/3361d9d38dd3145f3ee4bfe2c21dfb54ac2ae983))

## [0.11.2](https://github.com/martinohmann/hcl-rs/compare/v0.11.1...v0.11.2) (2022-12-31)


### Features

* **structure:** add `Structure::into_{attribute, block}` ([e927806](https://github.com/martinohmann/hcl-rs/commit/e927806e55035b1f3dc509686c1e6a8dd317249e))

## [0.11.1](https://github.com/martinohmann/hcl-rs/compare/v0.11.0...v0.11.1) (2022-12-29)


### Features

* **structure:** add `BlockLabel::as_str` ([#144](https://github.com/martinohmann/hcl-rs/issues/144)) ([75ade0b](https://github.com/martinohmann/hcl-rs/commit/75ade0bce3cf1b53acba4fc6177a412ac9dedfd2)), closes [#143](https://github.com/martinohmann/hcl-rs/issues/143)

## [0.11.0](https://github.com/martinohmann/hcl-rs/compare/v0.10.0...v0.11.0) (2022-12-25)


### ⚠ BREAKING CHANGES

* **serde:** `hcl::ser::Serializer` does not implement `serde::Serializer` anymore. Use `serializer.serialize(&value)` in places where `value.serialize(&mut serializer)` was used before.

### Features

* **de:** support expressions in custom types ([#139](https://github.com/martinohmann/hcl-rs/issues/139)) ([3fd68ec](https://github.com/martinohmann/hcl-rs/commit/3fd68ec70c7f2a30ef5e13782e473ca654b2cef8)), closes [#137](https://github.com/martinohmann/hcl-rs/issues/137)
* **serde:** use internal serialization to roundtrip crate types ([#135](https://github.com/martinohmann/hcl-rs/issues/135)) ([fbd555b](https://github.com/martinohmann/hcl-rs/commit/fbd555b457e3f97d443ffdd33b9ed15971aa4a2a))
* **ser:** support serializing blocks from custom types ([#140](https://github.com/martinohmann/hcl-rs/issues/140)) ([d97bceb](https://github.com/martinohmann/hcl-rs/commit/d97bceb3c76865012f5177e66c4767db9decaaf8)), closes [#138](https://github.com/martinohmann/hcl-rs/issues/138)


### Miscellaneous

* **clippy:** fix newly added lints ([e95e4d0](https://github.com/martinohmann/hcl-rs/commit/e95e4d0c7bb5b5c975738da3d69796ac37b64f0f))
* **deps:** bump dependencies to latest versions ([#141](https://github.com/martinohmann/hcl-rs/issues/141)) ([509e62e](https://github.com/martinohmann/hcl-rs/commit/509e62e41d3e574ec39b15199ed77bd0ec1cbf64))

## [0.10.0](https://github.com/martinohmann/hcl-rs/compare/v0.9.3...v0.10.0) (2022-11-17)


### ⚠ BREAKING CHANGES

* **eval:** the `Evaluate` implementation of `TemplateExpr` returns a `Value` instead of a `String` now to support interpolation unwrapping.

### Features

* **format:** add `prefer_ident_keys` to `FormatterBuilder` ([#134](https://github.com/martinohmann/hcl-rs/issues/134)) ([de48f5c](https://github.com/martinohmann/hcl-rs/commit/de48f5cf2c962526deb4d98081e3e7c1d51b315c)), closes [#132](https://github.com/martinohmann/hcl-rs/issues/132)
* **template:** implement `Serialize`/`Deserialize` for `Template` ([49e3bdd](https://github.com/martinohmann/hcl-rs/commit/49e3bdde85738c2ee0e8950acbf83a0e6ed7d7cf))


### Bug Fixes

* **eval:** correctly handle interpolation unwrapping ([85bed59](https://github.com/martinohmann/hcl-rs/commit/85bed59d3a5b37542bd0daaa577e1c07cc12ac7a))
* **format:** prevent double-escaping of template strings ([#133](https://github.com/martinohmann/hcl-rs/issues/133)) ([9d0d6b4](https://github.com/martinohmann/hcl-rs/commit/9d0d6b49e32716205d88f5688430cc591255281a)), closes [#131](https://github.com/martinohmann/hcl-rs/issues/131)
* **parser:** improve grammar for heredoc ([#130](https://github.com/martinohmann/hcl-rs/issues/130)) ([d366a32](https://github.com/martinohmann/hcl-rs/commit/d366a3228a186adfee7165374135c35a572538e0))


### Miscellaneous

* **lint:** make clippy more pedantic ([#129](https://github.com/martinohmann/hcl-rs/issues/129)) ([5a20055](https://github.com/martinohmann/hcl-rs/commit/5a200550c3b1c8da6794ef271a5baa1935d0c1fb))

## [0.9.3](https://github.com/martinohmann/hcl-rs/compare/v0.9.2...v0.9.3) (2022-11-07)


### Features

* **template:** implement `Format` for template types ([#123](https://github.com/martinohmann/hcl-rs/issues/123)) ([40bd497](https://github.com/martinohmann/hcl-rs/commit/40bd497783551c2c65849ae67b1ed31c8dc51529))


### Miscellaneous

* **ci:** add `clippy` job to `ci` workflow ([#124](https://github.com/martinohmann/hcl-rs/issues/124)) ([54e1ae2](https://github.com/martinohmann/hcl-rs/commit/54e1ae29cfda90795c483f9d699010f91b105550))

## [0.9.2](https://github.com/martinohmann/hcl-rs/compare/v0.9.1...v0.9.2) (2022-11-06)


### Features

* **expr:** add `TraversalBuilder` ([#121](https://github.com/martinohmann/hcl-rs/issues/121)) ([5e6daaa](https://github.com/martinohmann/hcl-rs/commit/5e6daaaea9689b2a950a9d0ec2b3cdbf79fc569f))
* **expr:** export `to_expression` function ([108a747](https://github.com/martinohmann/hcl-rs/commit/108a74706ddb479cdca8b6f49162326cf7db277d))
* **format:** add support for compact collection formatting ([#122](https://github.com/martinohmann/hcl-rs/issues/122)) ([5a12ff5](https://github.com/martinohmann/hcl-rs/commit/5a12ff50d08142c7a63c2498dcfc81c1277a621b))
* **macros:** add `value!` macro ([#120](https://github.com/martinohmann/hcl-rs/issues/120)) ([463ca92](https://github.com/martinohmann/hcl-rs/commit/463ca92b58473a61f5a5d31f24ba200a144a1ac0))
* **value:** add `ValueSerializer` and `to_value` ([#119](https://github.com/martinohmann/hcl-rs/issues/119)) ([fc57b6d](https://github.com/martinohmann/hcl-rs/commit/fc57b6d7dc0d919177dd1df2106ccebd06d97d9b))


### Bug Fixes

* **format:** separate object items with commas in compact mode ([36a7b09](https://github.com/martinohmann/hcl-rs/commit/36a7b09343d002a256ee766ea19150391a4a64c5))

## [0.9.1](https://github.com/martinohmann/hcl-rs/compare/v0.9.0...v0.9.1) (2022-11-02)


### Features

* re-export `Template` type at the crate root ([3fac02b](https://github.com/martinohmann/hcl-rs/commit/3fac02bdf96e72ddf461d4e498fd01798d0da930))


### Bug Fixes

* unclutter crate root exports ([#117](https://github.com/martinohmann/hcl-rs/issues/117)) ([d8a72bf](https://github.com/martinohmann/hcl-rs/commit/d8a72bf08cd54111ba76406d3c4bee477d0831b0))


### Miscellaneous

* **deps:** bump actions/cache from 3.0.9 to 3.0.11 ([#113](https://github.com/martinohmann/hcl-rs/issues/113)) ([4706b46](https://github.com/martinohmann/hcl-rs/commit/4706b46b21d7b49e5d07b27ce4505831bf4bfc4f))
* **deps:** update textwrap requirement from 0.15.0 to 0.16.0 ([#114](https://github.com/martinohmann/hcl-rs/issues/114)) ([9b4f5a4](https://github.com/martinohmann/hcl-rs/commit/9b4f5a41b11f18e1783f239786a3d91de5439780))
* **format:** remove ident validation ([#116](https://github.com/martinohmann/hcl-rs/issues/116)) ([d21083e](https://github.com/martinohmann/hcl-rs/commit/d21083e74c45e013dce5e77d994194ec0e25f8b2))
* move `Identifier` type to crate root ([#110](https://github.com/martinohmann/hcl-rs/issues/110)) ([c8e6af0](https://github.com/martinohmann/hcl-rs/commit/c8e6af08a6b9db7f05bad3b69b13ab0d9b98c9dc))
* move expression types into `expr` module ([#115](https://github.com/martinohmann/hcl-rs/issues/115)) ([6acba6c](https://github.com/martinohmann/hcl-rs/commit/6acba6c63b26ddf02055d4eb98da60dffda11765)), closes [#100](https://github.com/martinohmann/hcl-rs/issues/100)
* update description in `Cargo.toml` ([8fc4d9b](https://github.com/martinohmann/hcl-rs/commit/8fc4d9b17f15a04aecdd2d5e9ee4ee5881ed5c12))

## [0.9.0](https://github.com/martinohmann/hcl-rs/compare/v0.8.8...v0.9.0) (2022-10-27)


### ⚠ BREAKING CHANGES

* The type of `Expression`'s `Variable` variant changed from `Identifier` to `Variable`. You can create a variable from an identifier via `Variable::from(identifier)` or by using `Variable::new` and `Variable::sanitized`.
* The `From<Identifier>` implementation for `String` has been removed because it causes problems with trait bounds in `From` implementations for other types. Use `Identifier::into_inner` instead.
* The `Block` struct's `identifier` field type was changed from `String` to `Identifier`. Furthermore, the trait bound for the block identifier on `Block::new` changed from `Into<String>` to `Into<Identifier>`.
* The `Attribute` struct's `key` field type was changed from `String` to `Identifier`. Furthermore, the trait bound for the attribute identifier on `Attribute::new` changed from `Into<String>` to `Into<Identifier>`.
* `Identifier::new` is fallible now and the return value changed from `Identifier` to `Result<Identifier, Error>`. An infallible alternative is provided with `Identifier::sanitized`. The inner `String` field of `Identifier` is now private to prevent direct modification.

### Features

* add `Variable` type ([#108](https://github.com/martinohmann/hcl-rs/issues/108)) ([ee5a5c8](https://github.com/martinohmann/hcl-rs/commit/ee5a5c805d09d6d31b5f28b7b456f8e510aee8a4))
* expression and template evaluation ([#99](https://github.com/martinohmann/hcl-rs/issues/99)) ([ce0d229](https://github.com/martinohmann/hcl-rs/commit/ce0d2291433129b9fc9d9fbd6cad192efe00815d))
* implement `AsRef&lt;str&gt;` and `Borrow<str>` for `Identifier` ([0a616e1](https://github.com/martinohmann/hcl-rs/commit/0a616e154dc4f4b928bd774c4faf4791d29294e8))


### Bug Fixes

* change `Attribute` key field type to `Identifier` ([#106](https://github.com/martinohmann/hcl-rs/issues/106)) ([84e1538](https://github.com/martinohmann/hcl-rs/commit/84e1538569c8989262c74747de1f59b65d3110fd))
* change `Block` identifier field type to `Identifier` ([#107](https://github.com/martinohmann/hcl-rs/issues/107)) ([badce8a](https://github.com/martinohmann/hcl-rs/commit/badce8ace467d2913bf57d7025f06ff44c75e868))
* change type of `Expression::Variable` variant ([#109](https://github.com/martinohmann/hcl-rs/issues/109)) ([5e1501a](https://github.com/martinohmann/hcl-rs/commit/5e1501ae7cf3092e996c216f92f3d7e61ef58432))
* remove `From&lt;Identifier&gt;` for `String` ([32a94ff](https://github.com/martinohmann/hcl-rs/commit/32a94ff476071f3a2b942ee118effca19baf69ff))
* sanitize identifiers upon creation ([#105](https://github.com/martinohmann/hcl-rs/issues/105)) ([7b085d7](https://github.com/martinohmann/hcl-rs/commit/7b085d78cdd71415df4dfd13fa4fad91b9c988c2))


### Miscellaneous

* mark `BlockLabel::{string,identifier}` functions as deprecated ([86dda75](https://github.com/martinohmann/hcl-rs/commit/86dda757224fde937d06ececff99a88d617af534))
* mark `ObjectKey::identifier` function as deprecated ([d2f5f94](https://github.com/martinohmann/hcl-rs/commit/d2f5f94fd560ec5c7d9ada6e4bbeb817604fbafa))

## [0.8.8](https://github.com/martinohmann/hcl-rs/compare/v0.8.7...v0.8.8) (2022-10-22)


### Bug Fixes

* **parser:** don't swallow leading whitespace in template ([9422195](https://github.com/martinohmann/hcl-rs/commit/942219524812025b0245317accd0ea97e37a4e61))

## [0.8.7](https://github.com/martinohmann/hcl-rs/compare/v0.8.6...v0.8.7) (2022-10-22)


### Features

* add `hcl::from_body` ([c9f6a68](https://github.com/martinohmann/hcl-rs/commit/c9f6a6804760258fe06b6f0f07f69cc63d17efc9))
* implement `From<(bool, bool)>` for `StripMode` ([79f85c7](https://github.com/martinohmann/hcl-rs/commit/79f85c740faeccd8b530e6e639c3dff1e8c96476))
* implement `From<&str>` for `TemplateExpr` ([fb8023d](https://github.com/martinohmann/hcl-rs/commit/fb8023d29f469c74df18dc19a4154ff361bf8446))
* implement `From<ObjectKey>` for `Value` ([f10fd07](https://github.com/martinohmann/hcl-rs/commit/f10fd07ebaac56073530b9d06a5344e54880a30e))


### Miscellaneous

* **deps:** bump vecmap-rs from 0.1.3 to 0.1.7 ([392e1b6](https://github.com/martinohmann/hcl-rs/commit/392e1b679053daca6fc61598b96c0863c93ecf99))

## [0.8.6](https://github.com/martinohmann/hcl-rs/compare/v0.8.5...v0.8.6) (2022-10-14)


### Features

* **format:** add `to_string_unchecked` and `strict_mode` ([#94](https://github.com/martinohmann/hcl-rs/issues/94)) ([c840d6d](https://github.com/martinohmann/hcl-rs/commit/c840d6dd5647199b3f4d395d338800cc1fbf613b))

## [0.8.5](https://github.com/martinohmann/hcl-rs/compare/v0.8.4...v0.8.5) (2022-10-12)


### Bug Fixes

* **format:** identifiers starting with underscores are valid ([#92](https://github.com/martinohmann/hcl-rs/issues/92)) ([6144b44](https://github.com/martinohmann/hcl-rs/commit/6144b44ebe7a36e5aae8ccbbda59065d1d181e8d)), closes [#91](https://github.com/martinohmann/hcl-rs/issues/91)
* **format:** prevent subtract with overflow in compact mode ([#88](https://github.com/martinohmann/hcl-rs/issues/88)) ([7dd8e90](https://github.com/martinohmann/hcl-rs/commit/7dd8e90f0c97a2a2b2d549d4d667d87e44336920)), closes [#87](https://github.com/martinohmann/hcl-rs/issues/87)
* **parser:** greatly improve expression parsing performance ([#90](https://github.com/martinohmann/hcl-rs/issues/90)) ([a5b57ef](https://github.com/martinohmann/hcl-rs/commit/a5b57ef0be8727a030c089b476f1a5475d8bf30e)), closes [#82](https://github.com/martinohmann/hcl-rs/issues/82)

## [0.8.4](https://github.com/martinohmann/hcl-rs/compare/v0.8.3...v0.8.4) (2022-10-07)


### Bug Fixes

* **parser:** comma between list items is not optional ([9d97a02](https://github.com/martinohmann/hcl-rs/commit/9d97a026c6ba52b3808ba45952a4b2ced16fef9e))
* **parser:** panic when parsing `LegacyIndex` traversal operator ([#86](https://github.com/martinohmann/hcl-rs/issues/86)) ([f7e9f87](https://github.com/martinohmann/hcl-rs/commit/f7e9f87f9340afc11505cdff8b6846f1dd1b541b))
* **traversal:** deserialize splat operators as unit ([#84](https://github.com/martinohmann/hcl-rs/issues/84)) ([9ff1894](https://github.com/martinohmann/hcl-rs/commit/9ff1894196528254f056f04807af65b3bfa5b9e2)), closes [#81](https://github.com/martinohmann/hcl-rs/issues/81)

## [0.8.3](https://github.com/martinohmann/hcl-rs/compare/v0.8.2...v0.8.3) (2022-10-01)


### Miscellaneous

* **deps:** bump actions/cache from 3.0.8 to 3.0.9 ([#78](https://github.com/martinohmann/hcl-rs/issues/78)) ([2abc271](https://github.com/martinohmann/hcl-rs/commit/2abc27156e67f698e4d219f0759dce6a1a5da43e))
* **deps:** update criterion requirement from 0.3 to 0.4 ([#79](https://github.com/martinohmann/hcl-rs/issues/79)) ([ac023bd](https://github.com/martinohmann/hcl-rs/commit/ac023bda2afcfd142422292eca7f9a8fb099ae73))

## [0.8.2](https://github.com/martinohmann/hcl-rs/compare/v0.8.1...v0.8.2) (2022-09-30)


### Features

* implement `Display` for `{Unary,Binary}Operator` ([765ae4e](https://github.com/martinohmann/hcl-rs/commit/765ae4e6fba64d6ff27ac5619c978068106f7bcc))
* implement `From<Numbe>` for `Value` ([5bc621a](https://github.com/martinohmann/hcl-rs/commit/5bc621a715e325da6e071b0a314b56dc30c06dcc))

## [0.8.1](https://github.com/martinohmann/hcl-rs/compare/v0.8.0...v0.8.1) (2022-09-30)


### Features

* implement `Copy` for some types ([66fba96](https://github.com/martinohmann/hcl-rs/commit/66fba96d672900bf74061e422f12b95e3eac8e95))
* implement `Display` and `Deref` for `Identifier` ([c06cec0](https://github.com/martinohmann/hcl-rs/commit/c06cec0466398fc6578058953f21c9cfae6fddbe))
* implement `Eq` and `Display` for `Value` ([cfb41fe](https://github.com/martinohmann/hcl-rs/commit/cfb41fecefb677f97144292d33814480f859cb98))

## [0.8.0](https://github.com/martinohmann/hcl-rs/compare/v0.7.0...v0.8.0) (2022-09-29)


### ⚠ BREAKING CHANGES

* The `Number` type was changed from an `enum` to an opaque `struct`. Use `Number::from` to create a number from an integer. Furthermore, the `From` implementations for `f32` and `f64` were removed. Use the newly added `Number::from_f64` instead.
* The `RawExpression` and `String` variants of the `ObjectKey` enum were removed in favor of the newly added `Expression` variant. Furthermore the methods `Object::raw_expression` and `ObjectKey::string` were removed. Use `ObjectKey::from` instead.
* The underlying map implementation for the `Object<K, V>` type changed from `IndexMap<K, V>` to `VecMap<K, V>`. For the most common operations this is a drop-in replacement, but `VecMap` lacks some of the more exotic APIs the `IndexMap` provides.
* Heredocs and quoted strings containing template interpolations and/or template directives are not parsed as `Expression::String` anymore, but end up as `Expression::TemplateExpr` (which can be further parsed into the template elements via `Template::from_expr`) instead. Expressions of kind `Expression::String` are guaranteed to not include any templating anymore.

### Features

* add `Expression::Conditional` ([#68](https://github.com/martinohmann/hcl-rs/issues/68)) ([e953ce1](https://github.com/martinohmann/hcl-rs/commit/e953ce151a5069ed951c22a5b313413498d7a3a8))
* add `Expression::ElementAccess` ([#63](https://github.com/martinohmann/hcl-rs/issues/63)) ([1f8c4d8](https://github.com/martinohmann/hcl-rs/commit/1f8c4d8708c23db59676b843393aa8c522bf2ad2))
* add `Expression::ForExpr` ([#70](https://github.com/martinohmann/hcl-rs/issues/70)) ([522bc80](https://github.com/martinohmann/hcl-rs/commit/522bc80c81ebc71050108c711326754a41d7c3db))
* add `Expression::FuncCall` ([#64](https://github.com/martinohmann/hcl-rs/issues/64)) ([d660712](https://github.com/martinohmann/hcl-rs/commit/d6607125c6309fe42b217a02a6f871368c540b14))
* add `Expression::Operation` ([#69](https://github.com/martinohmann/hcl-rs/issues/69)) ([4961a7c](https://github.com/martinohmann/hcl-rs/commit/4961a7cedbb04fb692df4dcb8c8720b73064c5b0))
* add `Expression::SubExpr` ([#65](https://github.com/martinohmann/hcl-rs/issues/65)) ([e37f090](https://github.com/martinohmann/hcl-rs/commit/e37f090373a100ca943a1f22ae8712d37191162e))
* add `Expression::VariableExpr` ([#62](https://github.com/martinohmann/hcl-rs/issues/62)) ([2b6e81f](https://github.com/martinohmann/hcl-rs/commit/2b6e81f106600b0ff23afabb690e1f1a4b8fcedf))
* allow any HCL expression as object key ([#73](https://github.com/martinohmann/hcl-rs/issues/73)) ([e39496c](https://github.com/martinohmann/hcl-rs/commit/e39496c09c6aa884c6d64b785d63372a611d1d1f))
* implement `PartialOrd` for `Number` ([#75](https://github.com/martinohmann/hcl-rs/issues/75)) ([b520f4e](https://github.com/martinohmann/hcl-rs/commit/b520f4e17bc567e91cbdea08c5ff1864716981c5))
* implement arithmetic ops for `Number` ([8d677d6](https://github.com/martinohmann/hcl-rs/commit/8d677d645b8e6a40260deeec181dbf2b01d93ed2))
* implement template sub-language ([#60](https://github.com/martinohmann/hcl-rs/issues/60)) ([12e88a3](https://github.com/martinohmann/hcl-rs/commit/12e88a37608f1921107e4cec8e82ab11b74c4395))
* use `VecMap` for object expressions ([#72](https://github.com/martinohmann/hcl-rs/issues/72)) ([025cb68](https://github.com/martinohmann/hcl-rs/commit/025cb68053ff50ee8161a20b40aa43293e49159b))


### Bug Fixes

* allow `-` in identifiers ([be06f53](https://github.com/martinohmann/hcl-rs/commit/be06f53c78f0f0811c9f47dcc2c5f4dfc6171e90))
* always do `f64` division ([5591e22](https://github.com/martinohmann/hcl-rs/commit/5591e226454d870794f26bbf371fed5b9056d6a2))
* correctly handle `Expression::Null` in serializer ([ae74def](https://github.com/martinohmann/hcl-rs/commit/ae74def273daee36f785f2ae1ff4ab207b687497))
* correctly handle `Expression` variants in `ExpressionSerializer` ([#71](https://github.com/martinohmann/hcl-rs/issues/71)) ([8d89437](https://github.com/martinohmann/hcl-rs/commit/8d89437aa33c99ab1c914a8048e46bf393796f1d))
* prevent creation of infinite and NaN `Number` ([#74](https://github.com/martinohmann/hcl-rs/issues/74)) ([f751fc0](https://github.com/martinohmann/hcl-rs/commit/f751fc0dc29a7977728ed6010ae77d269eb7ab44))
* use correct deserializer for index element access ([#67](https://github.com/martinohmann/hcl-rs/issues/67)) ([f7bdd5c](https://github.com/martinohmann/hcl-rs/commit/f7bdd5c97a9c03f00e0e6440b42f4dd6bbbfbd63))


### Miscellaneous

* **deps:** bump vecmap-rs to 0.1.3 ([5670415](https://github.com/martinohmann/hcl-rs/commit/5670415bb043bdd7908d656d07b1e59af19a9a55))

## [0.7.0](https://github.com/martinohmann/hcl-rs/compare/v0.6.5...v0.7.0) (2022-09-01)


### ⚠ BREAKING CHANGES

* this breaks some public APIs.

### Bug Fixes

* derive `Eq` where suggested by clippy ([307cc2c](https://github.com/martinohmann/hcl-rs/commit/307cc2cc1883a163c9074e25f7ddd3f14bad73af))


### refactor

* move formatter into `format` module ([#55](https://github.com/martinohmann/hcl-rs/issues/55)) ([ec98979](https://github.com/martinohmann/hcl-rs/commit/ec989794876632fe466928c3759586da2d78ff13))


### Miscellaneous

* **deps:** bump actions/cache from 3.0.5 to 3.0.7 ([#53](https://github.com/martinohmann/hcl-rs/issues/53)) ([77e2c70](https://github.com/martinohmann/hcl-rs/commit/77e2c70143ec991164ea8e10628f0ed09f8bf062))
* **deps:** bump actions/cache from 3.0.7 to 3.0.8 ([#59](https://github.com/martinohmann/hcl-rs/issues/59)) ([f78e16e](https://github.com/martinohmann/hcl-rs/commit/f78e16e874f1c4352deac88d28df280f63ef022f))

## [0.6.5](https://github.com/martinohmann/hcl-rs/compare/v0.6.4...v0.6.5) (2022-08-19)


### Miscellaneous

* **deps:** bump actions/cache from 3.0.4 to 3.0.5 ([#48](https://github.com/martinohmann/hcl-rs/issues/48)) ([530fee4](https://github.com/martinohmann/hcl-rs/commit/530fee4f93dd4dd7b642bf1c0b719b4364166909))
* **deps:** bump pest from 2.1.3 to 2.2.1 ([#52](https://github.com/martinohmann/hcl-rs/issues/52)) ([555b7e4](https://github.com/martinohmann/hcl-rs/commit/555b7e42a32bfcd005f2d2761d7e5a94c6704098))

## [0.6.4](https://github.com/martinohmann/hcl-rs/compare/v0.6.3...v0.6.4) (2022-07-30)


### Bug Fixes

* parse negate operations on numbers as negative numbers ([#46](https://github.com/martinohmann/hcl-rs/issues/46)) ([74eb690](https://github.com/martinohmann/hcl-rs/commit/74eb6900068caac93e3a42d1ffaeced0a96005d0))


### Miscellaneous

* remove unused lifetimes ([62e75f0](https://github.com/martinohmann/hcl-rs/commit/62e75f08d77b98d3a2195f223d363147d6b39f69))

## [0.6.3](https://github.com/martinohmann/hcl-rs/compare/v0.6.2...v0.6.3) (2022-07-30)


### Bug Fixes

* `!` is not a logic operator ([5012034](https://github.com/martinohmann/hcl-rs/commit/50120341eab72ac42eb3f9b4bfe354562d9df5a5))
* correctly handle ExprTerm traversal ([#42](https://github.com/martinohmann/hcl-rs/issues/42)) ([8dc0f88](https://github.com/martinohmann/hcl-rs/commit/8dc0f88d039395cbd6e449dd94d06283bc0d88e3))
* disable flaky specsuite test ([cd19180](https://github.com/martinohmann/hcl-rs/commit/cd191806a8de0da8351f20dca841e3055c4f2d37))
* implement `deserialize_option` for `ValueDeserializer` ([#45](https://github.com/martinohmann/hcl-rs/issues/45)) ([2ddf40f](https://github.com/martinohmann/hcl-rs/commit/2ddf40fd54d7107c677c0a71cf22bef1efaf9edb))

## [0.6.2](https://github.com/martinohmann/hcl-rs/compare/v0.6.1...v0.6.2) (2022-07-13)


### Bug Fixes

* handle escaped newlines in heredocs ([#41](https://github.com/martinohmann/hcl-rs/issues/41)) ([82849f1](https://github.com/martinohmann/hcl-rs/commit/82849f1cb380524949cf208a79ab36cd052268c7))


### Miscellaneous

* **deps:** bump actions/cache from 3.0.3 to 3.0.4 ([#39](https://github.com/martinohmann/hcl-rs/issues/39)) ([1ba1301](https://github.com/martinohmann/hcl-rs/commit/1ba13015c4667cfebe0d1bc9b3f186815c89ee65))

## [0.6.1](https://github.com/martinohmann/hcl-rs/compare/v0.6.0...v0.6.1) (2022-06-12)


### Bug Fixes

* **macros:** use `structure!` macro inside of `body!` macro ([1d9a00c](https://github.com/martinohmann/hcl-rs/commit/1d9a00c114ab5ae16c50697d4ae0a1c1d5e91eba))

## [0.6.0](https://github.com/martinohmann/hcl-rs/compare/v0.5.2...v0.6.0) (2022-06-10)


### Features

* add `hcl::body!` macro and others ([#36](https://github.com/martinohmann/hcl-rs/issues/36)) ([987f8fc](https://github.com/martinohmann/hcl-rs/commit/987f8fc01c33a9cfae8d41dd29aca85cad2a3f80))

## [0.5.2](https://github.com/martinohmann/hcl-rs/compare/v0.5.1...v0.5.2) (2022-06-06)


### Bug Fixes

* unescape strings while parsing ([#34](https://github.com/martinohmann/hcl-rs/issues/34)) ([6fcc332](https://github.com/martinohmann/hcl-rs/commit/6fcc332cbbba02f7dd0d405543a92829ffa57b08))

## [0.5.1](https://github.com/martinohmann/hcl-rs/compare/v0.5.0...v0.5.1) (2022-06-04)


### Bug Fixes

* **serializer:** do not emit newline if block body is empty ([#32](https://github.com/martinohmann/hcl-rs/issues/32)) ([a36fa7d](https://github.com/martinohmann/hcl-rs/commit/a36fa7d263647c76725374d1feb63b0824b3789a))

## [0.5.0](https://github.com/martinohmann/hcl-rs/compare/v0.4.0...v0.5.0) (2022-06-03)


### Features

* add HCL serializer ([#30](https://github.com/martinohmann/hcl-rs/issues/30)) ([a9583c9](https://github.com/martinohmann/hcl-rs/commit/a9583c9a457d40b0e685d0e5864ca5146c41c6cc))

## [0.4.0](https://github.com/martinohmann/hcl-rs/compare/v0.3.3...v0.4.0) (2022-06-03)


### Features

* specialize deserialization for `hcl::Body` ([#24](https://github.com/martinohmann/hcl-rs/issues/24)) ([5581ccf](https://github.com/martinohmann/hcl-rs/commit/5581ccfcbef2ec6e231b33d089423e57cc59dfc7))
* **structure:** add `Expression` type ([#20](https://github.com/martinohmann/hcl-rs/issues/20)) ([2661f30](https://github.com/martinohmann/hcl-rs/commit/2661f309a57ed79bcf8a6744632589243e5f46fa))


### Bug Fixes

* **expression:** rename `Tuple` variant to `Array` ([72306f2](https://github.com/martinohmann/hcl-rs/commit/72306f27a164e2aa2e7febe6210ba4f586b4822e))


### Miscellaneous

* **deps:** bump actions/cache from 2 to 3.0.1 ([#18](https://github.com/martinohmann/hcl-rs/issues/18)) ([6c1ea15](https://github.com/martinohmann/hcl-rs/commit/6c1ea15c90860902d7eca178faec0527acf25ce2))
* **deps:** bump actions/cache from 3.0.1 to 3.0.2 ([#23](https://github.com/martinohmann/hcl-rs/issues/23)) ([d7198d9](https://github.com/martinohmann/hcl-rs/commit/d7198d9fa5895c6957f442f4397e379d435e0950))
* **deps:** bump actions/cache from 3.0.2 to 3.0.3 ([#28](https://github.com/martinohmann/hcl-rs/issues/28)) ([8065451](https://github.com/martinohmann/hcl-rs/commit/80654511a543a12e2c5ac0eb9e3d95429eff2697))

### [0.3.3](https://github.com/martinohmann/hcl-rs/compare/v0.3.2...v0.3.3) (2022-03-26)


### Bug Fixes

* correctly handle non-space indent in heredocs ([#15](https://github.com/martinohmann/hcl-rs/issues/15)) ([cc8a043](https://github.com/martinohmann/hcl-rs/commit/cc8a043ae0bfc0f522e1a427fe1984c1e9e519ab))

### [0.3.2](https://github.com/martinohmann/hcl-rs/compare/v0.3.1...v0.3.2) (2022-03-26)


### Bug Fixes

* heredocs must be newline-terminated ([#13](https://github.com/martinohmann/hcl-rs/issues/13)) ([2c7353e](https://github.com/martinohmann/hcl-rs/commit/2c7353e805d421070c05cc45032248184bd4852c))

### [0.3.1](https://github.com/martinohmann/hcl-rs/compare/v0.3.0...v0.3.1) (2022-03-25)


### Bug Fixes

* strip indent from `<<-` heredocs as defined in the HCL spec ([#11](https://github.com/martinohmann/hcl-rs/issues/11)) ([61a0ea9](https://github.com/martinohmann/hcl-rs/commit/61a0ea9f6ebdc3353e4221b21d632a5c84cef7a0))

## [0.3.0](https://github.com/martinohmann/hcl-rs/compare/v0.2.1...v0.3.0) (2022-03-25)


### Features

* add specsuite integration harness ([#9](https://github.com/martinohmann/hcl-rs/issues/9)) ([71e1205](https://github.com/martinohmann/hcl-rs/commit/71e1205836e17b469b106dc4af1d1fa7f47589b2))

### [0.2.1](https://github.com/martinohmann/hcl-rs/compare/v0.2.0...v0.2.1) (2022-03-23)


### Bug Fixes

* **parser:** fix heredoc handling to match HCL spec ([#8](https://github.com/martinohmann/hcl-rs/issues/8)) ([ff58d8c](https://github.com/martinohmann/hcl-rs/commit/ff58d8c025c30bc97203950042f1e7692c2ff38c))


### Miscellaneous

* **deps:** bump actions/checkout from 2 to 3 ([#6](https://github.com/martinohmann/hcl-rs/issues/6)) ([79e7ff2](https://github.com/martinohmann/hcl-rs/commit/79e7ff23ffdd3cd9c7ccb8aa68680d0906274bf1))

## [0.2.0](https://github.com/martinohmann/hcl-rs/compare/v0.1.0...v0.2.0) (2022-03-04)


### ⚠ BREAKING CHANGES

* flatten single block bodies (#2)

### Features

* `add_*`-from-iterator methods for `{Body,Block}Builder` ([62a5fd7](https://github.com/martinohmann/hcl-rs/commit/62a5fd77d419754a25745f9225e0012e06008b83))
* add `*_attribute` and `*_block` methods to `Structure` ([a2e18c1](https://github.com/martinohmann/hcl-rs/commit/a2e18c12ec9203a19228e6abea3f5d64d3b0028e))
* add `Body` iterators ([d08e55e](https://github.com/martinohmann/hcl-rs/commit/d08e55ee5ddc4c5cb80d85615f978b0a87ce3646))
* add `hcl::de::from_slice` ([5917cf3](https://github.com/martinohmann/hcl-rs/commit/5917cf3453925bdf3c04af09c1afba5681e6b02d))
* add `iter` and `iter_mut` for `Body` ([c1778fe](https://github.com/martinohmann/hcl-rs/commit/c1778feac31173299363d5d91aa7a7a97348d5b3))
* add `key` and `value` methods to `Attribute` ([10c9ea0](https://github.com/martinohmann/hcl-rs/commit/10c9ea0310cc04e07cbb29b25e4b70d8db8e964c))
* add getters for block members ([8e36bae](https://github.com/martinohmann/hcl-rs/commit/8e36baeeea34b251b8319716fd285430e523c57a))
* export `hcl::parse` ([#3](https://github.com/martinohmann/hcl-rs/issues/3)) ([3393de3](https://github.com/martinohmann/hcl-rs/commit/3393de34347a194abb2adcbbe84d1414bd01289b))


### Bug Fixes

* flatten single block bodies ([#2](https://github.com/martinohmann/hcl-rs/issues/2)) ([252a44c](https://github.com/martinohmann/hcl-rs/commit/252a44c3d6d0d88f2589865835d189c766be6727))
