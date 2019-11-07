//! Conventional Commit components.

use std::fmt;
use std::ops::Deref;

/// A single footer.
///
/// A footer is similar to a Git trailer, with the exception of not requiring
/// whitespace before newlines.
///
/// See: <https://git-scm.com/docs/git-interpret-trailers>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Footer<'a> {
    token: FooterToken<'a>,
    sep: FooterSeparator,
    value: FooterValue<'a>,
}

impl<'a> Footer<'a> {
    /// The token of the footer.
    pub const fn token(&self) -> FooterToken<'a> {
        self.token
    }

    /// The separator between the footer token and its value.
    pub const fn separator(&self) -> FooterSeparator {
        self.sep
    }

    /// The value of the footer.
    pub const fn value(&self) -> FooterValue<'a> {
        self.value
    }
}

impl<'a> From<(&'a str, &'a str, &'a str)> for Footer<'a> {
    fn from((token, sep, value): (&'a str, &'a str, &'a str)) -> Self {
        Self {
            token: FooterToken::new(token),
            sep: sep.into(),
            value: FooterValue::new(value),
        }
    }
}

/// The "simple footer" variant, for convenient access to the string slice
/// values of its components.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SimpleFooter<'a> {
    pub(crate) footer: &'a Footer<'a>,
}

impl<'a> SimpleFooter<'a> {
    /// The token of the footer.
    pub fn token(&self) -> &str {
        &*self.footer.token
    }

    /// The separator between the footer token and its value.
    pub fn separator(&self) -> &str {
        &*self.footer.sep
    }

    /// The value of the footer.
    pub fn value(&self) -> &str {
        &*self.footer.value
    }
}

/// The type of separator between the footer token and value.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum FooterSeparator {
    /// ": "
    ColonSpace,

    /// " #"
    SpacePound,

    #[doc(hidden)]
    __NonExhaustive,
}

impl Deref for FooterSeparator {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            FooterSeparator::ColonSpace => ": ",
            FooterSeparator::SpacePound => " #",
            FooterSeparator::__NonExhaustive => "",
        }
    }
}

impl fmt::Display for FooterSeparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl From<&str> for FooterSeparator {
    fn from(sep: &str) -> Self {
        match sep {
            ": " => FooterSeparator::ColonSpace,
            " #" => FooterSeparator::SpacePound,
            _ => unreachable!(),
        }
    }
}

macro_rules! components {
($($ty:ident),+) => (
    $(
        /// A component of the conventional commit.
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub struct $ty<'a>(&'a str);

        impl<'a> $ty<'a> {
            /// Create a $ty
            pub fn new(value: &'a str) -> Self {
                $ty(value)
            }
        }

        impl Deref for $ty<'_> {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl fmt::Display for $ty<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<'a> From<&'a str> for $ty<'a> {
            fn from(string: &'a str) -> Self {
                Self(string)
            }
        }
    )+
)
}

macro_rules! unicase_components {
    ($($ty:ident),+) => (
        $(
            /// A component of the conventional commit.
            #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
            pub struct $ty<'a>(unicase::UniCase<&'a str>);

            impl<'a> $ty<'a> {
                /// Create a $ty
                pub fn new(value: &'a str) -> Self {
                    $ty(unicase::UniCase::new(value))
                }
            }

            impl Deref for $ty<'_> {
                type Target = str;

                fn deref(&self) -> &Self::Target {
                    &self.0.into_inner()
                }
            }

            impl fmt::Display for $ty<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }

            impl<'a> From<&'a str> for $ty<'a> {
                fn from(string: &'a str) -> Self {
                    Self(unicase::UniCase::new(string))
                }
            }
        )+
    )
}

components![Description, Body, FooterValue];

unicase_components![Type, Scope, FooterToken];
