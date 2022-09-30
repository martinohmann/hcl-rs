# Changelog

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
