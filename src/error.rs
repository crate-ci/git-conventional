//! All errors related to Conventional Commits.

use std::fmt;

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

impl From<nom::Err<nom::error::VerboseError<&str>>> for Error {
    fn from(err: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        use nom::error::VerboseErrorKind::*;

        match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(err) | nom::Err::Failure(err) => match err.errors.last() {
                None => unreachable!("you found a bug!"),
                Some((_, kind)) => {
                    {};
                    match kind {
                        Context(string) => match *string {
                            "type" => Error::MissingType,
                            "scope_block" | "scope" => Error::InvalidScope,
                            "description" => Error::MissingDescription,
                            "body" => Error::InvalidBody,
                            "space" | "colon" | _ => Error::InvalidFormat,
                            }
                        Char(_) | Nom(_) => Error::InvalidFormat,
                    }
                }
            },
        }
    }
}
