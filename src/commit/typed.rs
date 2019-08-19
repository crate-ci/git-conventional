//! Conventional Commit implementations.
//! Conventional Commit implementations.
//! Conventional Commit implementations.

use crate::{Body, Commit, Description, Scope, Trailer, Type};

/// The strongly-typed variant of a commit.
pub trait Typed<'a> {
    /// The type of the commit.
    fn type_(&self) -> Type<'a>;

    /// The optional scope of the commit.
    fn scope(&self) -> Option<Scope<'a>>;

    /// The commit description.
    fn description(&self) -> Description<'a>;

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    fn body(&self) -> Option<Body<'a>>;

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

    /// Any Git trailers.
    ///
    /// See: <https://git-scm.com/docs/git-interpret-trailers>
    fn trailers(&self) -> &[Trailer<'_>];
}

impl<'a> Typed<'a> for Commit<'a> {
    fn type_(&self) -> Type<'a> {
        self.ty
    }

    fn scope(&self) -> Option<Scope<'a>> {
        self.scope
    }

    fn description(&self) -> Description<'a> {
        self.description
    }

    fn body(&self) -> Option<Body<'a>> {
        self.body
    }

    fn breaking(&self) -> bool {
        self.breaking
    }

    fn trailers(&self) -> &[Trailer<'_>] {
        &self.trailers
    }
}
