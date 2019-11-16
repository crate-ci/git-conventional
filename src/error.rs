//! All errors related to Conventional Commits.

use std::fmt;

/// The error returned when parsing a commit fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,

    commit: Option<String>,
}

impl Error {
    /// Create a new error from a `ErrorKind`.
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, commit: None }
    }

    pub(crate) fn with_nom(commit: &str, err: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        use nom::error::VerboseErrorKind::*;
        use ErrorKind::*;

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

    /// The kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self.kind {
            MissingType => f.write_str("missing type definition"),
            InvalidScope => f.write_str("invalid scope format"),
            MissingDescription => f.write_str("missing commit description"),
            InvalidBody => f.write_str("invalid body format"),
            InvalidFormat => f.write_str("invalid commit format"),
            __NonExhaustive => unreachable!("__NonExhaustive is unused"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// All possible error kinds returned when parsing a conventional commit.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
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

    #[doc(hidden)]
    __NonExhaustive,
}
