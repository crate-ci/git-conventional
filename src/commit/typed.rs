//! Conventional Commit implementations.
//! Conventional Commit implementations.
//! Conventional Commit implementations.

use crate::typed::{Body, Description, Footer, Scope, Type};
use crate::Commit;

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
    fn footers(&self) -> &[Footer<'_>];
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

    fn footers(&self) -> &[Footer<'_>] {
        &self.footers
    }
}
