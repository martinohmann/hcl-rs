# Contributing

Contributions to `hcl-rs` are very welcome and I'm happy about everything that
improves the quality of this crate.

To avoid friction, please read the following guidelines before contributing to
this project:

- Please always open an issue for new features or breaking changes before
  sending a PR to discuss it.
- Non-breaking bugfixes and documentation fixes don't require an associated
  issue before sending a PR.
- Any non-trivial code change should also include a decent amount of tests.
- If possible, please add a regression test as part of a bug fix.
- This project uses [Conventional
  Commits](https://www.conventionalcommits.org/en/v1.0.0/). Please use a
  suitable prefix in PR titles (e.g. `feat`, `fix`, `chore`, `docs`). Breaking
  changes should be marked as such by adding a `!` after the type/scope. You
  don't need to use the Conventional Commit format for every commit that is
  included in your PR. PRs will be squash merged.
- Make sure to run `cargo test`, `cargo clippy` and `rustfmt` before submitting
  a PR.
