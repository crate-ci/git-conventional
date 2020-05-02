//! All errors related to Conventional Commits.

use std::fmt;

/// The error returned when parsing a commit fails.
pub struct Error {
    kind: ErrorKind,

    context: Option<Box<dyn fmt::Display>>,
    commit: Option<String>,
}

impl Error {
    /// Create a new error from a `ErrorKind`.
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: None,
            commit: None,
        }
    }

    pub(crate) fn with_nom(commit: &str, err: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        use nom::error::VerboseErrorKind::*;
        use ErrorKind::*;

        let kind = match err {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(err) | nom::Err::Failure(err) => match err.errors.last() {
                None => unreachable!("you found a bug!"),
                Some((_, kind)) => match kind {
                    Context(string) => match *string {
                        crate::parser::TYPE => MissingType,
                        crate::parser::SCOPE => InvalidScope,
                        crate::parser::DESCRIPTION => MissingDescription,
                        crate::parser::BODY => InvalidBody,
                        crate::parser::FORMAT | _ => InvalidFormat,
                    },
                    Char(_) | Nom(_) => InvalidFormat,
                },
            },
        };

        Self {
            kind,
            context: None,
            commit: Some(commit.to_owned()),
        }
    }

    pub(crate) fn set_context(mut self, context: Box<dyn fmt::Display>) -> Self {
        self.context = Some(context);
        self
    }

    /// The kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("context", &self.context.as_ref().map(|s| s.to_string()))
            .field("commit", &self.commit)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(context) = self.context.as_ref() {
            write!(f, "{}: {}", self.kind, context)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// All possible error kinds returned when parsing a conventional commit.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The commit type is missing from the commit message.
    MissingType,

    /// The scope has an invalid format.
    InvalidScope,

    /// The description of the commit is missing.
    MissingDescription,

    /// The body of the commit has an invalid format.
    InvalidBody,

    /// The footer of the commit has an invalid format.
    InvalidFooter,

    /// Any other part of the commit does not conform to the conventional commit
    /// spec.
    InvalidFormat,

    #[doc(hidden)]
    __NonExhaustive,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorKind::MissingType => "Missing type definition",
            ErrorKind::InvalidScope => "Invalid scope format",
            ErrorKind::MissingDescription => "Missing commit description",
            ErrorKind::InvalidBody => "invalid body format",
            ErrorKind::InvalidFooter => "invalid body footer",
            ErrorKind::InvalidFormat => "invalid commit format",
            ErrorKind::__NonExhaustive => unreachable!("__NonExhaustive is unused"),
        };
        f.write_str(s)
    }
}
