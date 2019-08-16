//! A parser library for the [Conventional Commit] specification.
//!
//! [conventional commit]: https://www.conventionalcommits.org
//!
//! # Example
//!
//! ```rust
//! use conventional::{Commit, Error};
//! use std::str::FromStr;
//!
//! fn main() -> Result<(), Error> {
//!     let message = "\
//!     docs(example): add tested usage example
//!
//!     This example is tested using Rust's doctest capabilities. Having this
//!     example helps people understand how to use the parser.
//!
//!     BREAKING CHANGE: Going from nothing to something, meaning anyone doing
//!     nothing before suddenly has something to do. That sounds like a change
//!     in your break.
//!     ";
//!
//!     let commit = Commit::new(message)?;
//!
//!     assert_eq!(commit.type_(), "docs");
//!     assert_eq!(commit.scope(), Some("example"));
//!     assert_eq!(commit.description(), "add tested usage example");
//!     assert!(commit.body().unwrap().contains("helps people understand"));
//!     assert!(commit.breaking_change().unwrap().contains("That sounds like a change"));
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

pub mod error;
mod parser;

pub use error::Error;

use std::fmt;
use std::ops::Deref;
use nom::error::VerboseError;
use parser::parse;

/// A conventional commit.
#[derive(Debug)]
pub struct Commit<'a> {
    ty: Type<'a>,
    scope: Option<Scope<'a>>,
    description: Description<'a>,
    body: Option<Body<'a>>,
    breaking_change: Option<BreakingChange<'a>>,
}

macro_rules! components {
    ($($ty:ident),+) => (
        $(
            /// A component of the conventional commit.
            #[derive(Debug, Clone, Eq, PartialEq, Hash)]
            struct $ty<'a>(&'a str);

            impl Deref for $ty<'_> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl fmt::Display for $ty<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl<'a> From<&'a str> for $ty<'a> {
                fn from(string: &'a str) -> Self {
                    Self(string)
                }
            }
        )+
    )
}

components![Type, Scope, Description, Body, BreakingChange];

impl<'a> Commit<'a> {
    /// Create a new Conventional Commit based on the provided commit message
    /// string.
    ///
    /// # Errors
    ///
    /// This function returns an error if the commit does not conform to the
    /// Conventional Commit specification.
    pub fn new(string: &'a str) -> Result<Self, Error> {
        let (_, (ty, scope, description, body, breaking_change)) =
            parse::<VerboseError<&'a str>>(string)?;

        Ok(Self {
            ty: ty.into(),
            scope: scope.map(Into::into),
            description: description.into(),
            body: body.map(Into::into),
            breaking_change: breaking_change.map(Into::into),
        })
    }

    /// The type of the commit.
    pub fn type_(&self) -> &str {
        self.ty.trim()
    }

    /// The optional scope of the commit.
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_ref().map(Deref::deref).map(str::trim)
    }

    /// The commit description.
    pub fn description(&self) -> &str {
        self.description.trim()
    }

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    pub fn body(&self) -> Option<&str> {
        self.body.as_ref().map(Deref::deref).map(str::trim)
    }

    /// The text discussing any breaking changes.
    pub fn breaking_change(&self) -> Option<&str> {
        self.breaking_change
            .as_ref()
            .map(Deref::deref)
            .map(str::trim)
    }
}

impl fmt::Display for Commit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_())?;

        if let Some(scope) = &self.scope() {
            f.write_fmt(format_args!("({})", scope))?;
        }

        f.write_fmt(format_args!(": {}", self.description))?;

        if let Some(body) = &self.body() {
            f.write_fmt(format_args!("\n\n{}", body))?;
        }

        if let Some(breaking_change) = &self.breaking_change() {
            f.write_fmt(format_args!("\n\nBREAKING CHANGE: {}", breaking_change))?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simple_commit() {
        let commit = Commit::new("my type(my scope): hello world").unwrap();

        assert_eq!("my type", commit.type_());
        assert_eq!(Some("my scope"), commit.scope());
        assert_eq!("hello world", commit.description());
    }

    #[test]
    fn test_valid_complex_commit() {
        let commit = "chore: improve changelog readability\n
                      \n
                      Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit \
                      easier to parse while reading.\n
                      \n
                      BREAKING CHANGE: Just kidding!";

        let commit = Commit::new(commit).unwrap();

        assert_eq!("chore", commit.type_());
        assert_eq!(None, commit.scope());
        assert_eq!("improve changelog readability", commit.description());
        assert_eq!(
            Some(
                "Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit \
                 easier to parse while reading."
            ),
            commit.body()
        );
        assert_eq!(Some("Just kidding!"), commit.breaking_change());
    }

    #[test]
    fn test_missing_type() {
        let err = Commit::new("").unwrap_err();

        assert_eq!(Error::MissingType, err);
    }
}
