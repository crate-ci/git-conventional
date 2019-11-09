//! The conventional commit type and its simple, and typed implementations.

pub(crate) mod simple;
pub(crate) mod typed;

use crate::component::{Body, Description, Footer, Scope, Type};
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
    footers: Vec<Footer<'a>>,
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
        let (ty, scope, breaking, description, body, footers) =
            parse::<VerboseError<&'a str>>(string).map_err(|err| (string, err))?;

        let breaking = breaking.is_some()
            || footers
                .iter()
                .any(|(k, _, _)| k == &"BREAKING CHANGE" || k == &"BREAKING-CHANGE");
        let footers: Result<Vec<_>, Error> = footers
            .into_iter()
            .map(|(k, s, v)| Ok(Footer::new(k.into(), s.parse()?, v.into())))
            .collect();
        let footers = footers?;

        Ok(Self {
            ty: ty.into(),
            scope: scope.map(Into::into),
            description: description.into(),
            body: body.map(Into::into),
            breaking,
            footers,
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

        for t in self.footers() {
            write!(f, "\n\n{}{}{}", t.token(), t.separator(), t.value())?;
        }

        Ok(())
    }
}
