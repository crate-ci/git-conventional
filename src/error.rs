//! All errors related to Conventional Commits.

use std::fmt;

/// The error returned when parsing a commit fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// The kind of error.
    pub kind: Kind,

    commit: Option<String>,
}

impl Error {
    /// Create a new error from a `Kind`.
    pub(crate) fn new(kind: Kind) -> Self {
        Self { kind, commit: None }
    }
}

/// All possible error kinds returned when parsing a conventional commit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
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
        use Kind::*;

        match self.kind {
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

impl<'a> From<(&'a str, nom::Err<nom::error::VerboseError<&'a str>>)> for Error {
    fn from((commit, err): (&'a str, nom::Err<nom::error::VerboseError<&'a str>>)) -> Self {
        use nom::error::VerboseErrorKind::*;
        use Kind::*;

        let kind = match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(err) | nom::Err::Failure(err) => match err.errors.last() {
                None => unreachable!("you found a bug!"),
                Some((_, kind)) => {
                    {};
                    match kind {
                        Context(string) => match *string {
                            "type" => MissingType,
                            "scope_block" | "scope" => InvalidScope,
                            "description" => MissingDescription,
                            "body" => InvalidBody,
                            "space" | "colon" | _ => InvalidFormat,
                        },
                        Char(_) | Nom(_) => InvalidFormat,
                    }
                }
            },
        };

        Self {
            commit: Some(commit.to_owned()),
            kind,
        }
    }
}
