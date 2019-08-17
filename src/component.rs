//! Conventional Commit components.

use std::fmt;
use std::ops::Deref;

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

components![Type, Scope, Description, Body, BreakingChange];
