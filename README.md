<div align="center">

<h1><code>conventional::Commit</code></h1>

[![Latest Crate Version](https://img.shields.io/crates/v/conventional.svg?logo=rust&label=version&logoColor=white&colorB=brightgreen)](https://crates.io/crates/conventional "The latest released version on crates.io.")
[![Library Documentation](https://docs.rs/conventional/badge.svg)](https://docs.rs/conventional "The online documentation at docs.rs.")
[![Discord Chat](https://img.shields.io/discord/477552212156088320.svg?logo=discord&label=discord%20chat&logoColor=white)](https://discord.gg/Kc4qZWE "Ask a question or just enjoy your stay!")

<br />
<strong>A Rust parser library for the <a href="https://www.conventionalcommits.org">Conventional Commit</a> spec.</strong>
<br />
<br />

</div>

### Quick Start

1. Add the crate to your `Cargo.toml`:

   ```shell
   cargo install cargo-edit

   cargo add conventional
   ```

2. Import the `Commit` type and the `Simple` trait to parse a commit string, and
   query its different components as string slices:

   ```rust
   use conventional::{Commit, Simple as _};

   let commit = Commit::new("feat(conventional commit): this is it!").unwrap();

   assert_eq!("feat", commit.type_());
   assert_eq!("conventional commit", commit.scope());
   assert_eq!("this is it!", commit.description());
   assert_eq!(None, commit.body());
   ```

3. Upgrade to `Typed` components for strongly typed access:

   ```rust
   use conventional::{Commit, Typed as _};

   let commit = Commit::new("feat(conventional commit): this is it!").unwrap();

   assert_eq!(Type("feat"), commit.type_());
   ```

4. Check out tools like [**Jilu**] for an example of library usage.

[latest specification]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/#specification
[**Jilu**]: https://github.com/rustic-games/jilu
