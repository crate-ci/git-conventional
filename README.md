# `code>conventional::Commit`

[![Build Status](https://dev.azure.com/crate-ci/crate-ci/_apis/build/status/git-conventional?branchName=master)](https://dev.azure.com/crate-ci/crate-ci/_build/latest?definitionId=12&branchName=master)
[![codecov](https://codecov.io/gh/crate-ci/git-conventional/branch/master/graph/badge.svg)](https://codecov.io/gh/crate-ci/git-conventional)
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/git-conventional.svg)
[![Crates Status](https://img.shields.io/crates/v/git-conventional.svg)](https://crates.io/crates/git-conventional)

> A Rust parser library for the [Conventional Commit](https://www.conventionalcommits.org) spec.

## Quick Start

1. Add the crate to your `Cargo.toml`:

   ```shell
   cargo install cargo-edit

   cargo add git_conventional
   ```

2. Parse a commit and lookup what you need

   ```rust
   let commit = git_conventional::Commit::parse("feat(conventional commit): this is it!").unwrap();

   assert_eq!(commit.type_(), git_conventional::FEAT);
   assert_eq!(commit.scope().unwrap(), "conventional commit");
   assert_eq!(commit.description(), "this is it!");
   assert_eq!(commit.body(), None);
   ```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/git-conventional
[Documentation]: https://docs.rs/git-conventional
