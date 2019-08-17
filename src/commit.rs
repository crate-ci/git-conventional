//! The conventional commit type and its simple, and typed implementations.

pub mod simple;
pub mod typed;

use crate::component::{Body, BreakingChange, Description, Scope, Type};
use crate::error::Error;
use crate::parser::parse;
use nom::error::VerboseError;
use std::fmt;

/// A conventional commit.
#[derive(Clone, Debug)]
pub struct Commit<'a> {
    ty: Type<'a>,
    scope: Option<Scope<'a>>,
    description: Description<'a>,
    body: Option<Body<'a>>,
    breaking: bool,
    breaking_change: Option<BreakingChange<'a>>,
}

impl<'a> Commit<'a> {
    /// Create a new Conventional Commit based on the provided commit message
    /// string.
    ///
    /// # Errors
    ///
    /// This function returns an error if the commit does not conform to the
    /// Conventional Commit specification.
    pub fn new(string: &'a str) -> Result<Self, Error> {
        let (_, (ty, scope, breaking, description, body, breaking_change)) =
            parse::<VerboseError<&'a str>>(string).map_err(|err| (string, err))?;

        Ok(Self {
            ty: ty.into(),
            scope: scope.map(Into::into),
            description: description.into(),
            body: body.map(Into::into),
            breaking,
            breaking_change: breaking_change.map(Into::into),
        })
    }
}

impl fmt::Display for Commit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::Simple;

        f.write_str(self.type_())?;

        if let Some(scope) = &self.scope() {
            f.write_fmt(format_args!("({})", scope))?;
        }

        f.write_fmt(format_args!(": {}", &self.description()))?;

        if let Some(body) = &self.body() {
            f.write_fmt(format_args!("\n\n{}", body))?;
        }

        if let Some(breaking_change) = &self.breaking_change() {
            f.write_fmt(format_args!("\n\nBREAKING CHANGE: {}", breaking_change))?;
        }

        Ok(())
    }
}
