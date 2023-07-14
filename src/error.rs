//! All errors related to Conventional Commits.

use std::fmt;

/// The error returned when parsing a commit fails.
pub struct Error {
    kind: ErrorKind,

    context: Option<Box<dyn fmt::Display + Send + Sync>>,
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

    pub(crate) fn with_nom(
        commit: &str,
        err: winnow::error::ParseError<&str, winnow::error::ContextError>,
    ) -> Self {
        use winnow::error::StrContext;
        use ErrorKind::*;

        let mut kind = InvalidFormat;
        for context in err.inner().context() {
            kind = match context {
                StrContext::Label(string) => match *string {
                    crate::parser::SUMMARY => MissingType,
                    crate::parser::TYPE => MissingType,
                    crate::parser::SCOPE => InvalidScope,
                    crate::parser::DESCRIPTION => MissingDescription,
                    crate::parser::BODY => InvalidBody,
                    _ => kind,
                },
                _ => kind,
            };
        }

        Self {
            kind,
            context: None,
            commit: Some(commit.to_owned()),
        }
    }

    pub(crate) fn set_context(mut self, context: Box<dyn fmt::Display + Send + Sync>) -> Self {
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
#[non_exhaustive]
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
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorKind::MissingType => {
                "Missing type in the commit summary, expected `type: description`"
            }
            ErrorKind::InvalidScope => {
                "Incorrect scope syntax in commit summary, expected `type(scope): description`"
            }
            ErrorKind::MissingDescription => {
                "Missing description in commit summary, expected `type: description`"
            }
            ErrorKind::InvalidBody => "Incorrect body syntax",
            ErrorKind::InvalidFooter => "Incorrect footer syntax",
            ErrorKind::InvalidFormat => "Incorrect conventional commit format",
        };
        f.write_str(s)
    }
}
