# Changelog

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
