//! A parser library for the [Conventional Commit] specification.
//!
//! [conventional commit]: https://www.conventionalcommits.org
//!
//! # Example
//!
//! ```rust
//! use indoc::indoc;
//! use conventional::{Commit, Error, Simple as _};
//! use std::str::FromStr;
//!
//! fn main() -> Result<(), Error> {
//!     let message = indoc!("
//!         docs(example)!: add tested usage example
//!
//!         This example is tested using Rust's doctest capabilities. Having this
//!         example helps people understand how to use the parser.
//!
//!         BREAKING CHANGE: Going from nothing to something, meaning anyone doing
//!         nothing before suddenly has something to do. That sounds like a change
//!         in your break.
//!
//!         Co-Authored-By: Lisa Simpson <lisa@simpsons.fam>
//!         Closes #12
//!     ");
//!
//!     let commit = Commit::new(message)?;
//!
//!     // You can access all components of the header.
//!     assert_eq!(commit.type_(), "docs");
//!     assert_eq!(commit.scope(), Some("example"));
//!     assert_eq!(commit.description(), "add tested usage example");
//!
//!     // And the free-form commit body.
//!     assert!(commit.body().unwrap().contains("helps people understand"));
//!
//!     // If a commit is marked with a bang (`!`) OR has a trailer with the key
//!     // "BREAKING CHANGE", it is considered a "breaking" commit.
//!     assert!(commit.breaking());
//!
//!     // You can access each trailer individually.
//!     assert!(commit.trailers().get(0).unwrap().value().contains("That sounds like a change"));
//!
//!     // Trailers provide access to their key and value.
//!     assert_eq!(commit.trailers().get(1).unwrap().key(), "Co-Authored-By");
//!     assert_eq!(commit.trailers().get(1).unwrap().value(), "Lisa Simpson <lisa@simpsons.fam>");
//!
//!     // Two types of separators are supported, regular ": ", and " #":
//!     assert_eq!(commit.trailers().get(2).unwrap().separator(), " #");
//!     assert_eq!(commit.trailers().get(2).unwrap().value(), "12");
//!     # Ok(())
//! }
//! ```

#![deny(
    clippy::all,
    clippy::cargo,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::multiple_inherent_impl,
    clippy::nursery,
    clippy::option_unwrap_used,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::result_unwrap_used,
    clippy::unimplemented,
    clippy::use_debug,
    clippy::wildcard_enum_match_arm,
    clippy::wrong_pub_self_convention,
    deprecated_in_future,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    warnings
)]
#![doc(html_root_url = "https://docs.rs/conventional")]

mod commit;
mod component;
mod error;
mod parser;

pub use commit::{simple::Simple, typed::Typed, Commit};
pub use component::SimpleTrailer;
pub use error::{Error, Kind as ErrorKind};

/// Strictly-typed accessors for a `Commit`.
pub mod typed {
    pub use super::component::{
        Body, Description, Scope, Trailer, TrailerKey, TrailerSeparator, TrailerValue, Type,
    };
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
mod tests {
    use super::{Commit, ErrorKind, Simple as _};
    use indoc::indoc;

    #[test]
    fn test_valid_simple_commit() {
        let commit = Commit::new("my type(my scope): hello world").unwrap();

        assert_eq!("my type", commit.type_());
        assert_eq!(Some("my scope"), commit.scope());
        assert_eq!("hello world", commit.description());
    }

    #[test]
    fn test_breaking_change() {
        let commit = Commit::new("feat!: this is a breaking change").unwrap();
        assert_eq!("feat", commit.type_());
        assert!(commit.breaking());

        let commit = Commit::new(indoc!(
            "feat: message

            BREAKING CHANGE: breaking change"
        ))
        .unwrap();

        assert_eq!("feat", commit.type_());
        assert_eq!(
            "breaking change",
            &*commit.trailers().get(0).unwrap().value()
        );
        assert!(commit.breaking());
    }

    #[test]
    fn test_valid_complex_commit() {
        let commit = indoc! {"
            chore: improve changelog readability

            Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit
            easier to parse while reading.

            BREAKING CHANGE: Just kidding!
        "};

        let commit = Commit::new(commit).unwrap();

        assert_eq!("chore", commit.type_());
        assert_eq!(None, commit.scope());
        assert_eq!("improve changelog readability", commit.description());
        assert_eq!(
            Some(indoc!(
                "Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit
                 easier to parse while reading."
            )),
            commit.body()
        );
        assert_eq!("Just kidding!", &*commit.trailers().get(0).unwrap().value());
    }

    #[test]
    fn test_missing_type() {
        let err = Commit::new("").unwrap_err();

        assert_eq!(ErrorKind::MissingType, err.kind);
    }

    mod typed {
        use crate::{component::*, Commit, Typed as _};

        #[test]
        fn test_typed_commit() {
            let commit = Commit::new("my type(my scope): hello world").unwrap();

            assert_eq!(Type("my type"), commit.type_());
            assert_eq!(Some(Scope("my scope")), commit.scope());
            assert_eq!(Description("hello world"), commit.description());
        }
    }
}
