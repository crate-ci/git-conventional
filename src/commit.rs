//! The conventional commit type and its simple, and typed implementations.

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use nom::error::VerboseError;

use crate::parser::parse;
use crate::{Error, ErrorKind};

const BREAKING_PHRASE: &str = "BREAKING CHANGE";
const BREAKING_ARROW: &str = "BREAKING-CHANGE";

/// A conventional commit.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Commit<'a> {
    ty: Type<'a>,
    scope: Option<Scope<'a>>,
    description: &'a str,
    body: Option<&'a str>,
    breaking: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    breaking_description: Option<&'a str>,
    footers: Vec<Footer<'a>>,
}

impl<'a> Commit<'a> {
    /// Create a new Conventional Commit based on the provided commit message
    /// string.
    ///
    /// # Errors
    ///
    /// This function returns an error if the commit does not conform to the
    /// Conventional Commit specification.
    pub fn parse(string: &'a str) -> Result<Self, Error> {
        let (ty, scope, breaking, description, body, footers) =
            parse::<VerboseError<&'a str>>(string).map_err(|err| Error::with_nom(string, err))?;

        let breaking_description = footers
            .iter()
            .filter_map(|(k, _, v)| (k == &BREAKING_PHRASE || k == &BREAKING_ARROW).then(|| *v))
            .next()
            .or_else(|| breaking.then(|| description));
        let breaking = breaking_description.is_some();
        let footers: Result<Vec<_>, Error> = footers
            .into_iter()
            .map(|(k, s, v)| Ok(Footer::new(FooterToken::new_unchecked(k), s.parse()?, v)))
            .collect();
        let footers = footers?;

        Ok(Self {
            ty: Type::new_unchecked(ty),
            scope: scope.map(Scope::new_unchecked),
            description,
            body,
            breaking,
            breaking_description,
            footers,
        })
    }

    /// The type of the commit.
    pub fn type_(&self) -> Type<'a> {
        self.ty
    }

    /// The optional scope of the commit.
    pub fn scope(&self) -> Option<Scope<'a>> {
        self.scope
    }

    /// The commit description.
    pub fn description(&self) -> &'a str {
        self.description
    }

    /// The commit body, containing a more detailed explanation of the commit
    /// changes.
    pub fn body(&self) -> Option<&'a str> {
        self.body
    }

    /// A flag to signal that the commit contains breaking changes.
    ///
    /// This flag is set either when the commit has an exclamation mark after
    /// the message type and scope, e.g.:
    /// ```text
    /// feat(scope)!: this is a breaking change
    /// ```
    ///
    /// Or when the `BREAKING CHANGE: ` footer is defined:
    /// ```text
    /// feat: my commit description
    ///
    /// BREAKING CHANGE: this is a breaking change
    /// ```
    pub fn breaking(&self) -> bool {
        self.breaking
    }

    /// Explanation for the breaking change.
    ///
    /// Note: if no `BREAKING CHANGE` footer is provided, the `description` is expected to describe
    /// the breaking change.
    pub fn breaking_description(&self) -> Option<&'a str> {
        self.breaking_description
    }

    /// Any footer.
    ///
    /// A footer is similar to a Git trailer, with the exception of not
    /// requiring whitespace before newlines.
    ///
    /// See: <https://git-scm.com/docs/git-interpret-trailers>
    pub fn footers(&self) -> &[Footer<'a>] {
        &self.footers
    }
}

impl fmt::Display for Commit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_().as_str())?;

        if let Some(scope) = &self.scope() {
            f.write_fmt(format_args!("({})", scope))?;
        }

        f.write_fmt(format_args!(": {}", &self.description()))?;

        if let Some(body) = &self.body() {
            f.write_fmt(format_args!("\n\n{}", body))?;
        }

        for t in self.footers() {
            write!(f, "\n\n{}{}{}", t.token(), t.separator(), t.value())?;
        }

        Ok(())
    }
}

/// A single footer.
///
/// A footer is similar to a Git trailer, with the exception of not requiring
/// whitespace before newlines.
///
/// See: <https://git-scm.com/docs/git-interpret-trailers>
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Footer<'a> {
    token: FooterToken<'a>,
    sep: FooterSeparator,
    value: &'a str,
}

impl<'a> Footer<'a> {
    /// Piece together a footer.
    pub const fn new(token: FooterToken<'a>, sep: FooterSeparator, value: &'a str) -> Self {
        Self { token, sep, value }
    }

    /// The token of the footer.
    pub const fn token(&self) -> FooterToken<'a> {
        self.token
    }

    /// The separator between the footer token and its value.
    pub const fn separator(&self) -> FooterSeparator {
        self.sep
    }

    /// The value of the footer.
    pub const fn value(&self) -> &'a str {
        self.value
    }

    /// A flag to signal that the footer describes a breaking change.
    pub fn breaking(&self) -> bool {
        self.token.breaking()
    }
}

/// The type of separator between the footer token and value.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum FooterSeparator {
    /// ":"
    Value,

    /// " #"
    Ref,
}

impl FooterSeparator {
    /// Access `str` representation of FooterSeparator
    pub fn as_str(self) -> &'static str {
        match self {
            FooterSeparator::Value => ":",
            FooterSeparator::Ref => " #",
        }
    }
}

impl Deref for FooterSeparator {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<&'_ str> for FooterSeparator {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl fmt::Display for FooterSeparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl FromStr for FooterSeparator {
    type Err = Error;

    fn from_str(sep: &str) -> Result<Self, Self::Err> {
        match sep {
            ":" => Ok(FooterSeparator::Value),
            " #" => Ok(FooterSeparator::Ref),
            _ => {
                Err(Error::new(ErrorKind::InvalidFooter)
                    .set_context(Box::new(format!("{:?}", sep))))
            }
        }
    }
}

macro_rules! unicase_components {
    ($($ty:ident),+) => (
        $(
            /// A component of the conventional commit.
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub struct $ty<'a>(unicase::UniCase<&'a str>);

            impl<'a> $ty<'a> {
                /// See `parse` for ensuring the data is valid.
                pub const fn new_unchecked(value: &'a str) -> Self {
                    $ty(unicase::UniCase::unicode(value))
                }

                /// Access `str` representation
                pub fn as_str(&self) -> &'a str {
                    &self.0.into_inner()
                }
            }

            impl Deref for $ty<'_> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    self.as_str()
                }
            }

            impl PartialEq<&'_ str> for $ty<'_> {
                fn eq(&self, other: &&str) -> bool {
                    *self == $ty::new_unchecked(*other)
                }
            }

            impl fmt::Display for $ty<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            #[cfg(feature = "serde")]
            impl serde::Serialize for $ty<'_> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_str(self)
                }
            }
        )+
    )
}

unicase_components![Type, Scope, FooterToken];

impl<'a> Type<'a> {
    /// Parse a `str` into a `Type`.
    pub fn parse(sep: &'a str) -> Result<Self, Error> {
        let (i, t) = crate::parser::type_(sep).map_err(|err| Error::with_nom(sep, err))?;
        if !i.is_empty() {
            return Err(Error::new(ErrorKind::InvalidFormat));
        }
        Ok(Type::new_unchecked(t))
    }
}

/// Common commit types
impl Type<'static> {
    /// Commit type when introducing new features (correlates with `minor` in semver)
    pub const FEAT: Type<'static> = Type::new_unchecked("feat");
    /// Commit type when patching a bug (correlates with `patch` in semver)
    pub const FIX: Type<'static> = Type::new_unchecked("fix");
    /// Possible commit type when reverting changes.
    pub const REVERT: Type<'static> = Type::new_unchecked("revert");
    /// Possible commit type for changing documentation.
    pub const DOCS: Type<'static> = Type::new_unchecked("docs");
    /// Possible commit type for changing code style.
    pub const STYLE: Type<'static> = Type::new_unchecked("style");
    /// Possible commit type for refactoring code structure.
    pub const REFACTOR: Type<'static> = Type::new_unchecked("refactor");
    /// Possible commit type for performance optimizations.
    pub const PERF: Type<'static> = Type::new_unchecked("perf");
    /// Possible commit type for addressing tests.
    pub const TEST: Type<'static> = Type::new_unchecked("test");
    /// Possible commit type for other things.
    pub const CHORE: Type<'static> = Type::new_unchecked("chore");
}

impl<'a> Scope<'a> {
    /// Parse a `str` into a `Scope`.
    pub fn parse(sep: &'a str) -> Result<Self, Error> {
        let (i, t) = crate::parser::scope(sep).map_err(|err| Error::with_nom(sep, err))?;
        if !i.is_empty() {
            return Err(Error::new(ErrorKind::InvalidScope));
        }
        Ok(Scope::new_unchecked(t))
    }
}

impl<'a> FooterToken<'a> {
    /// Parse a `str` into a `FooterToken`.
    pub fn parse(sep: &'a str) -> Result<Self, Error> {
        let (i, t) = crate::parser::token(sep).map_err(|err| Error::with_nom(sep, err))?;
        if !i.is_empty() {
            return Err(Error::new(ErrorKind::InvalidScope));
        }
        Ok(FooterToken::new_unchecked(t))
    }

    /// A flag to signal that the footer describes a breaking change.
    pub fn breaking(&self) -> bool {
        self == &BREAKING_PHRASE || self == &BREAKING_ARROW
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ErrorKind;
    use indoc::indoc;
    #[cfg(feature = "serde")]
    use serde_test::Token;

    #[test]
    fn test_valid_simple_commit() {
        let commit = Commit::parse("type(my scope): hello world").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope().unwrap(), "my scope");
        assert_eq!(commit.description(), "hello world");
    }

    #[test]
    fn test_trailing_whitespace_without_body() {
        let commit = Commit::parse("type(my scope): hello world\n\n\n").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope().unwrap(), "my scope");
        assert_eq!(commit.description(), "hello world");
    }

    #[test]
    fn test_trailing_1_nl() {
        let commit = Commit::parse("type: hello world\n").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "hello world");
    }

    #[test]
    fn test_trailing_2_nl() {
        let commit = Commit::parse("type: hello world\n\n").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "hello world");
    }

    #[test]
    fn test_trailing_3_nl() {
        let commit = Commit::parse("type: hello world\n\n\n").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "hello world");
    }

    #[test]
    fn test_parenthetical_statement() {
        let commit = Commit::parse("type: hello world (#1)").unwrap();

        assert_eq!(commit.type_(), "type");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "hello world (#1)");
    }

    #[test]
    fn test_issue_12_case_1() {
        // Looks like it was test_trailing_2_nl that triggered this to fail originally
        let commit = Commit::parse("chore: add .hello.txt (#1)\n\n").unwrap();

        assert_eq!(commit.type_(), "chore");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "add .hello.txt (#1)");
    }

    #[test]
    fn test_issue_12_case_2() {
        // Looks like it was test_trailing_2_nl that triggered this to fail originally
        let commit = Commit::parse("refactor: use fewer lines (#3)\n\n").unwrap();

        assert_eq!(commit.type_(), "refactor");
        assert_eq!(commit.scope(), None);
        assert_eq!(commit.description(), "use fewer lines (#3)");
    }

    #[test]
    fn test_breaking_change() {
        let commit = Commit::parse("feat!: this is a breaking change").unwrap();
        assert_eq!(Type::FEAT, commit.type_());
        assert!(commit.breaking());
        assert_eq!(
            commit.breaking_description(),
            Some("this is a breaking change")
        );

        let commit = Commit::parse(indoc!(
            "feat: message

            BREAKING CHANGE: breaking change"
        ))
        .unwrap();
        assert_eq!(Type::FEAT, commit.type_());
        assert_eq!(
            "breaking change",
            &*commit.footers().get(0).unwrap().value()
        );
        assert!(commit.breaking());
        assert_eq!(commit.breaking_description(), Some("breaking change"));

        let commit = Commit::parse(indoc!(
            "fix: message

            BREAKING-CHANGE: it's broken"
        ))
        .unwrap();
        assert_eq!(Type::FIX, commit.type_());
        assert_eq!("it's broken", &*commit.footers().get(0).unwrap().value());
        assert!(commit.breaking());
        assert_eq!(commit.breaking_description(), Some("it's broken"));
    }

    #[test]
    fn test_valid_complex_commit() {
        let commit = indoc! {"
            chore: improve changelog readability

            Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit
            easier to parse while reading.

            BREAKING CHANGE: Just kidding!
        "};

        let commit = Commit::parse(commit).unwrap();

        assert_eq!(Type::CHORE, commit.type_());
        assert_eq!(None, commit.scope());
        assert_eq!("improve changelog readability", commit.description());
        assert_eq!(
            Some(indoc!(
                "Change date notation from YYYY-MM-DD to YYYY.MM.DD to make it a tiny bit
                 easier to parse while reading."
            )),
            commit.body()
        );
        assert_eq!("Just kidding!", &*commit.footers().get(0).unwrap().value());
    }

    #[test]
    fn test_missing_type() {
        let err = Commit::parse("").unwrap_err();

        assert_eq!(ErrorKind::MissingType, err.kind());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_commit_serialize() {
        let commit = Commit::parse("type(my scope): hello world").unwrap();
        serde_test::assert_ser_tokens(
            &commit,
            &[
                Token::Struct {
                    name: "Commit",
                    len: 6,
                },
                Token::Str("ty"),
                Token::Str("type"),
                Token::Str("scope"),
                Token::Some,
                Token::Str("my scope"),
                Token::Str("description"),
                Token::Str("hello world"),
                Token::Str("body"),
                Token::None,
                Token::Str("breaking"),
                Token::Bool(false),
                Token::Str("footers"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
