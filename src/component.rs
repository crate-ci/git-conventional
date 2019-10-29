//! Conventional Commit components.

use std::fmt;
use std::ops::Deref;

/// A single Git trailer.
///
/// See: <https://git-scm.com/docs/git-interpret-trailers>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Trailer<'a> {
    key: TrailerKey<'a>,
    sep: TrailerSeparator,
    value: TrailerValue<'a>,
}

impl<'a> Trailer<'a> {
    /// The key of the trailer.
    pub const fn key(&self) -> TrailerKey<'a> {
        self.key
    }

    /// The separator between the trailer key and its value.
    pub const fn separator(&self) -> TrailerSeparator {
        self.sep
    }

    /// The value of the trailer.
    pub const fn value(&self) -> TrailerValue<'a> {
        self.value
    }
}

impl<'a> From<(&'a str, &'a str, &'a str)> for Trailer<'a> {
    fn from((key, sep, value): (&'a str, &'a str, &'a str)) -> Self {
        Self {
            key: TrailerKey(key),
            sep: sep.into(),
            value: TrailerValue(value),
        }
    }
}

/// The "simple trailer" variant, for convenient access to the string slice
/// values of its components.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SimpleTrailer<'a> {
    pub(crate) trailer: &'a Trailer<'a>,
}

impl<'a> SimpleTrailer<'a> {
    /// The key of the trailer.
    pub fn key(&self) -> &str {
        &*self.trailer.key
    }

    /// The separator between the trailer key and its value.
    pub fn separator(&self) -> &str {
        &*self.trailer.sep
    }

    /// The value of the trailer.
    pub fn value(&self) -> &str {
        &*self.trailer.value
    }
}

/// The type of separator between the trailer key and value.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TrailerSeparator {
    /// ": "
    ColonSpace,

    /// " #"
    SpacePound,
}

impl Deref for TrailerSeparator {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            TrailerSeparator::ColonSpace => ": ",
            TrailerSeparator::SpacePound => " #",
        }
    }
}

impl fmt::Display for TrailerSeparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrailerSeparator::ColonSpace => f.write_str(": "),
            TrailerSeparator::SpacePound => f.write_str(" #"),
        }
    }
}

impl From<&str> for TrailerSeparator {
    fn from(sep: &str) -> Self {
        match sep {
            ": " => TrailerSeparator::ColonSpace,
            " #" => TrailerSeparator::SpacePound,
            _ => unreachable!(),
        }
    }
}

macro_rules! components {
    ($($ty:ident),+) => (
        $(
            /// A component of the conventional commit.
            #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
            pub struct $ty<'a>(pub &'a str);

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

components![Type, Scope, Description, Body, TrailerKey, TrailerValue];
