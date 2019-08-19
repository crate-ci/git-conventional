//! Conventional Commit implementations.

use crate::Commit;
use std::ops::Deref;

/// The weakly-typed variant of a commit.
pub trait Simple {
    /// The type of the commit.
    fn type_(&self) -> &str;

    /// The optional scope of the commit.
    fn scope(&self) -> Option<&str>;

    /// The commit description.
    fn description(&self) -> &str;

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    fn body(&self) -> Option<&str>;

    /// The text discussing any breaking changes.
    fn breaking_change(&self) -> Option<&str>;

    /// A flag to signal that the commit contains breaking changes.
    ///
    /// This flag is set either when the commit has an exclamation mark after
    /// the message type and scope, e.g.:
    ///
    ///   feat(scope)!: this is a breaking change
    ///   feat!: this is a breaking change
    ///
    /// Or when the `BREAKING CHANGE: ` trailer is defined:
    ///
    ///   feat: my commit description
    ///
    ///   BREAKING CHANGE: this is a breaking change
    fn breaking(&self) -> bool;
}

impl Simple for Commit<'_> {
    fn type_(&self) -> &str {
        &self.ty
    }

    fn scope(&self) -> Option<&str> {
        self.scope.as_ref().map(Deref::deref)
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn body(&self) -> Option<&str> {
        self.body.as_ref().map(Deref::deref)
    }

    fn breaking(&self) -> bool {
        self.breaking
    }

    fn breaking_change(&self) -> Option<&str> {
        self.breaking_change.as_ref().map(Deref::deref)
    }
}
