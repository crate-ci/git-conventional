# `code>conventional::Commit`

[![Build Status](https://dev.azure.com/crate-ci/crate-ci/_apis/build/status/git-conventional?branchName=master)](https://dev.azure.com/crate-ci/crate-ci/_build/latest?definitionId=5&branchName=master)
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

2. Import the `Commit` type and the `Simple` trait to parse a commit string, and
   query its different components as string slices:

   ```rust
   use git_conventional::{Commit, Simple as _};

   let commit = Commit::new("feat(conventional commit): this is it!").unwrap();

   assert_eq!("feat", commit.type_());
   assert_eq!(Some("conventional commit"), commit.scope());
   assert_eq!("this is it!", commit.description());
   assert_eq!(None, commit.body());
   ```

3. Upgrade to `Typed` components for strongly typed access:

   ```rust
   use git_conventional::{Commit, typed::Type, Typed as _};

   let commit = Commit::new("feat(conventional commit): this is it!").unwrap();

   assert_eq!(Type::new("feat"), commit.type_());
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
