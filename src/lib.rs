//! A parser library for the [Conventional Commit] specification.
//!
//! [conventional commit]: https://www.conventionalcommits.org
//!
//! # Example
//!
//! ```rust
//! use indoc::indoc;
//!
//! let message = indoc!("
//!     docs(example)!: add tested usage example
//!
//!     This example is tested using Rust's doctest capabilities. Having this
//!     example helps people understand how to use the parser.
//!
//!     BREAKING CHANGE: Going from nothing to something, meaning anyone doing
//!     nothing before suddenly has something to do. That sounds like a change
//!     in your break.
//!
//!     Co-Authored-By: Lisa Simpson <lisa@simpsons.fam>
//!     Closes #12
//! ");
//!
//! let commit = git_conventional::Commit::parse(message).unwrap();
//!
//! // You can access all components of the subject.
//! assert_eq!(commit.type_(), git_conventional::Type::DOCS);
//! assert_eq!(commit.scope().unwrap(), "example");
//! assert_eq!(commit.description(), "add tested usage example");
//!
//! // And the free-form commit body.
//! assert!(commit.body().unwrap().contains("helps people understand"));
//!
//! // If a commit is marked with a bang (`!`) OR has a footer with the key
//! // "BREAKING CHANGE", it is considered a "breaking" commit.
//! assert!(commit.breaking());
//!
//! // You can access each footer individually.
//! assert!(commit.footers()[0].value().contains("That sounds like a change"));
//!
//! // Footers provide access to their token and value.
//! assert_eq!(commit.footers()[1].token(), "Co-Authored-By");
//! assert_eq!(commit.footers()[1].value(), "Lisa Simpson <lisa@simpsons.fam>");
//!
//! // Two types of separators are supported, regular ": ", and " #":
//! assert_eq!(commit.footers()[2].separator(), " #");
//! assert_eq!(commit.footers()[2].value(), "12");
//! ```

#![warn(missing_docs)]

mod commit;
mod error;
mod lines;
mod parser;

pub use commit::{Commit, Footer, FooterSeparator, FooterToken, Scope, Type};
pub use error::{Error, ErrorKind};

doc_comment::doctest!("../README.md");
