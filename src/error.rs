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
