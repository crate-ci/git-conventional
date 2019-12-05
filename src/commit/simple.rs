//! Conventional Commit implementations.

use crate::{Commit, SimpleFooter};

/// The weakly-typed variant of a commit.
pub trait Simple<'a> {
    /// The type of the commit.
    fn type_(&self) -> &'a str;

    /// The optional scope of the commit.
    fn scope(&self) -> Option<&'a str>;

    /// The commit description.
    fn description(&self) -> &'a str;

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    fn body(&self) -> Option<&'a str>;

    /// A flag to signal that the commit contains breaking changes.
    ///
    /// This flag is set either when the commit has an exclamation mark after
    /// the message type and scope, e.g.:
    ///
    ///   feat(scope)!: this is a breaking change
    ///   feat!: this is a breaking change
    ///
    /// Or when the `BREAKING CHANGE: ` footer is defined:
    ///
    ///   feat: my commit description
    ///
    ///   BREAKING CHANGE: this is a breaking change
    fn breaking(&self) -> bool;

    /// Any footer.
    ///
    /// A footer is similar to a Git trailer, with the exception of not
    /// requiring whitespace before newlines.
    ///
    /// See: <https://git-scm.com/docs/git-interpret-trailers>
    fn footers(&self) -> Vec<SimpleFooter<'a>>;
}

impl<'a> Simple<'a> for Commit<'a> {
    fn type_(&self) -> &'a str {
        self.ty.as_str()
    }

    fn scope(&self) -> Option<&'a str> {
        self.scope.as_ref().map(|s| s.as_str())
    }

    fn description(&self) -> &'a str {
        self.description.as_str()
    }

    fn body(&self) -> Option<&'a str> {
        self.body.as_ref().map(|s| s.as_str())
    }

    fn breaking(&self) -> bool {
        self.breaking
    }

    fn footers(&self) -> Vec<SimpleFooter<'a>> {
        self.footers
            .iter()
            .map(|footer| SimpleFooter { footer: *footer })
            .collect::<Vec<_>>()
    }
}
