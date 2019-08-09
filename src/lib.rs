//! A parser library for the [Conventional Commit] specification.
//!
//! [conventional commit]: https://www.conventionalcommits.org
//!
//! # Example
//!
//! ```rust
//! use conventional_commit::{ConventionalCommit, Error};
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
//!     let commit = ConventionalCommit::from_str(message)?;
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
#![doc(html_root_url = "https://docs.rs/conventional-commit")]

use itertools::Itertools;
use std::fmt;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

/// A conventional commit.
#[derive(Debug)]
pub struct ConventionalCommit {
    ty: String,
    scope: Option<String>,
    description: String,
    body: Option<String>,
    breaking_change: Option<String>,
}

impl ConventionalCommit {
    /// The type of the commit.
    pub fn type_(&self) -> &str {
        self.ty.trim()
    }

    /// The optional scope of the commit.
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_ref().map(String::as_str).map(str::trim)
    }

    /// The commit description.
    pub fn description(&self) -> &str {
        self.description.trim()
    }

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    pub fn body(&self) -> Option<&str> {
        self.body.as_ref().map(String::as_str).map(str::trim)
    }

    /// The text discussing any breaking changes.
    pub fn breaking_change(&self) -> Option<&str> {
        self.breaking_change
            .as_ref()
            .map(String::as_str)
            .map(str::trim)
    }
}

impl fmt::Display for ConventionalCommit {
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

impl FromStr for ConventionalCommit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Error::*;

        // Example:
        //
        // chore(changelog): improve changelog readability
        //
        // Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny
        // bit easier to parse while reading.
        let mut chars = UnicodeSegmentation::graphemes(s, true).peekable();

        // ex: "chore"
        let ty: String = chars
            .peeking_take_while(|&c| c != "(" && c != ":")
            .collect();
        if ty.is_empty() {
            return Err(MissingType);
        }

        // ex: "changelog"
        let mut scope: Option<String> = None;
        if chars.peek() == Some(&"(") {
            let _ = scope.replace(chars.peeking_take_while(|&c| c != ")").skip(1).collect());
            chars = chars.dropping(1);
        }

        if chars.by_ref().take(2).collect::<String>() != ": " {
            return Err(InvalidFormat);
        }

        // ex: "improve changelog readability"
        let description: String = chars.peeking_take_while(|&c| c != "\n").collect();
        if description.is_empty() {
            return Err(MissingDescription);
        }

        let other: String = chars.collect::<String>().trim().to_owned();

        // ex: "Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a
        //      tiny bit easier to parse while reading."
        let (body, breaking_change) = if other.is_empty() {
            (None, None)
        } else {
            let mut data = other
                .splitn(2, "BREAKING CHANGE:")
                .map(|s| s.trim().to_owned());

            (data.next(), data.next())
        };

        Ok(Self {
            ty,
            scope,
            description,
            body,
            breaking_change,
        })
    }
}

/// All possible errors returned when parsing a conventional commit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The commit type is missing from the commit message.
    MissingType,

    /// The scope has an invalid format.
    InvalidScope,

    /// The description of the commit is missing.
    MissingDescription,

    /// The body of the commit has an invalid format.
    InvalidBody,

    /// Any other part of the commit does not conform to the conventional commit
    /// spec.
    InvalidFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            MissingType => f.write_str("missing type definition"),
            InvalidScope => f.write_str("invalid scope format"),
            MissingDescription => f.write_str("missing commit description"),
            InvalidBody => f.write_str("invalid body format"),
            InvalidFormat => f.write_str("invalid commit format"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simple_commit() {
        let commit = ConventionalCommit::from_str("my type(my scope): hello world").unwrap();

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

        let commit = ConventionalCommit::from_str(commit).unwrap();

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
        let err = ConventionalCommit::from_str("").unwrap_err();

        assert_eq!(Error::MissingType, err);
    }
}
