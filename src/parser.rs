#![allow(clippy::let_unit_value)] // for clarify and to ensure the right type is selected

use std::str;

use winnow::ascii::line_ending;
use winnow::combinator::alt;
use winnow::combinator::repeat;
use winnow::combinator::trace;
use winnow::combinator::{cut_err, eof, fail, opt, peek};
use winnow::combinator::{delimited, preceded, terminated};
use winnow::error::{AddContext, ErrMode, ErrorKind, ParserError, StrContext};
use winnow::prelude::{PResult, Parser};
use winnow::stream::Stream as _;
use winnow::token::{take, take_till, take_while};

type CommitDetails<'a> = (
    &'a str,
    Option<&'a str>,
    bool,
    &'a str,
    Option<&'a str>,
    Vec<(&'a str, &'a str, &'a str)>,
);

pub(crate) fn parse<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<CommitDetails<'a>, E> {
    message.parse_next(i)
}

// <CR>              ::= "0x000D"
// <LF>              ::= "0x000A"
// <newline>         ::= [<CR>], <LF>
fn is_line_ending(c: char) -> bool {
    c == '\n' || c == '\r'
}

// <parens>          ::= "(" | ")"
fn is_parens(c: char) -> bool {
    c == '(' || c == ')'
}

// <ZWNBSP>          ::= "U+FEFF"
// <TAB>             ::= "U+0009"
// <VT>              ::= "U+000B"
// <FF>              ::= "U+000C"
// <SP>              ::= "U+0020"
// <NBSP>            ::= "U+00A0"
// /* See: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-white-space */
// <USP>             ::= "Any other Unicode 'Space_Separator' code point"
// /* Any non-newline whitespace: */
// <whitespace>      ::= <ZWNBSP> | <TAB> | <VT> | <FF> | <SP> | <NBSP> | <USP>
fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

fn whitespace<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    take_while(0.., is_whitespace).parse_next(i)
}

// <message>         ::= <summary>, <newline>+, <body>, (<newline>+, <footer>)*
//                    |  <summary>, (<newline>+, <footer>)*
//                    |  <summary>, <newline>*
pub(crate) fn message<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<CommitDetails<'a>, E> {
    trace("message", move |i: &mut &'a str| {
        let summary =
            terminated(trace("summary", summary), alt((line_ending, eof))).parse_next(i)?;
        let (type_, scope, breaking, description) = summary;

        // The body MUST begin one blank line after the description.
        let _ = alt((line_ending, eof))
            .context(StrContext::Label(BODY))
            .parse_next(i)?;

        let _extra: () = repeat(0.., line_ending).parse_next(i)?;

        let body = opt(body).parse_next(i)?;

        let footers = repeat(0.., footer).parse_next(i)?;

        let _: () = repeat(0.., line_ending).parse_next(i)?;

        Ok((type_, scope, breaking.is_some(), description, body, footers))
    })
    .parse_next(i)
}

// <type>            ::= <any UTF8-octets except newline or parens or ":" or "!:" or whitespace>+
pub(crate) fn type_<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace(
        "type",
        take_while(1.., |c: char| {
            !is_line_ending(c) && !is_parens(c) && c != ':' && c != '!' && !is_whitespace(c)
        })
        .context(StrContext::Label(TYPE)),
    )
    .parse_next(i)
}

pub(crate) const TYPE: &str = "type";

// <scope>           ::= <any UTF8-octets except newline or parens>+
pub(crate) fn scope<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace(
        "scope",
        take_while(1.., |c: char| !is_line_ending(c) && !is_parens(c))
            .context(StrContext::Label(SCOPE)),
    )
    .parse_next(i)
}

pub(crate) const SCOPE: &str = "scope";

// /* "!" should be added to the AST as a <breaking-change> node with the value "!" */
// <summary>         ::= <type>, "(", <scope>, ")", ["!"], ":", <whitespace>*, <text>
//                    |  <type>, ["!"], ":", <whitespace>*, <text>
#[allow(clippy::type_complexity)]
fn summary<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<(&'a str, Option<&'a str>, Option<&'a str>, &'a str), E> {
    trace(
        "summary",
        (
            type_,
            opt(delimited('(', cut_err(scope), ')')),
            opt(exclamation_mark),
            preceded(
                (':', whitespace),
                text.context(StrContext::Label(DESCRIPTION)),
            ),
        ),
    )
    .context(StrContext::Label(SUMMARY))
    .parse_next(i)
}

pub(crate) const SUMMARY: &str = "SUMMARY";
pub(crate) const DESCRIPTION: &str = "description";

// <text>            ::= <any UTF8-octets except newline>*
fn text<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace("text", take_till(1.., is_line_ending)).parse_next(i)
}

fn body<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace("body", move |i: &mut &'a str| {
        if i.is_empty() {
            let start = i.checkpoint();
            let err = E::from_error_kind(i, ErrorKind::Eof);
            let err = err.add_context(i, &start, StrContext::Label(BODY));
            return Err(ErrMode::Backtrack(err));
        }

        let mut offset = 0;
        let mut prior_is_empty = true;
        for line in crate::lines::LinesWithTerminator::new(i) {
            if prior_is_empty
                && peek::<_, _, E, _>((token, separator))
                    .parse_peek(line.trim_end())
                    .is_ok()
            {
                break;
            }
            prior_is_empty = line.trim().is_empty();

            offset += line.chars().count();
        }
        if offset == 0 {
            fail::<_, (), _>(i)?;
        }

        take(offset).map(str::trim_end).parse_next(i)
    })
    .parse_next(i)
}

pub(crate) const BODY: &str = "body";

// <footer>          ::= <token>, <separator>, <whitespace>*, <value>
fn footer<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<(&'a str, &'a str, &'a str), E> {
    trace(
        "footer",
        (token, separator, whitespace, value).map(|(ft, s, _, fv)| (ft, s, fv)),
    )
    .parse_next(i)
}

// <token>           ::= <breaking-change>
//                    |  <type>
pub(crate) fn token<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace("token", alt(("BREAKING CHANGE", type_))).parse_next(i)
}

// <separator>       ::= ":" | " #"
fn separator<'a, E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    trace("sep", alt((":", " #"))).parse_next(i)
}

pub(crate) fn value<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    if i.is_empty() {
        let start = i.checkpoint();
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = err.add_context(i, &start, StrContext::Label("value"));
        return Err(ErrMode::Cut(err));
    }

    let mut offset = 0;
    for (i, line) in crate::lines::LinesWithTerminator::new(i).enumerate() {
        if 0 < i
            && peek::<_, _, E, _>((token, separator))
                .parse_peek(line.trim_end())
                .is_ok()
        {
            break;
        }

        offset += line.chars().count();
    }

    take(offset).map(str::trim_end).parse_next(i)
}

fn exclamation_mark<
    'a,
    E: ParserError<&'a str> + AddContext<&'a str, StrContext> + std::fmt::Debug,
>(
    i: &mut &'a str,
) -> PResult<&'a str, E> {
    "!".context(StrContext::Label(BREAKER)).parse_next(i)
}

pub(crate) const BREAKER: &str = "exclamation_mark";

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use super::*;

    use winnow::error::ContextError;

    mod message {
        use super::*;
        #[test]
        fn errors() {
            let mut p = message::<ContextError>;

            let input = "Hello World";
            let err = p.parse(input).unwrap_err();
            let err = crate::Error::with_nom(input, err);
            assert_eq!(err.to_string(), crate::ErrorKind::MissingType.to_string());

            let input = "fix Improved error messages\n";
            let err = p.parse(input).unwrap_err();
            let err = crate::Error::with_nom(input, err);
            assert_eq!(err.to_string(), crate::ErrorKind::MissingType.to_string());
        }
    }

    mod summary {
        use super::*;

        #[test]
        fn test_type() {
            let mut p = type_::<ContextError>;

            // valid
            assert_eq!(p.parse_peek("foo").unwrap(), ("", "foo"));
            assert_eq!(p.parse_peek("Foo").unwrap(), ("", "Foo"));
            assert_eq!(p.parse_peek("FOO").unwrap(), ("", "FOO"));
            assert_eq!(p.parse_peek("fOO").unwrap(), ("", "fOO"));
            assert_eq!(p.parse_peek("foo2bar").unwrap(), ("", "foo2bar"));
            assert_eq!(p.parse_peek("foo-bar").unwrap(), ("", "foo-bar"));
            assert_eq!(p.parse_peek("foo bar").unwrap(), (" bar", "foo"));
            assert_eq!(p.parse_peek("foo: bar").unwrap(), (": bar", "foo"));
            assert_eq!(p.parse_peek("foo!: bar").unwrap(), ("!: bar", "foo"));
            assert_eq!(p.parse_peek("foo(bar").unwrap(), ("(bar", "foo"));
            assert_eq!(p.parse_peek("foo ").unwrap(), (" ", "foo"));

            // invalid
            assert!(p.parse_peek("").is_err());
            assert!(p.parse_peek(" ").is_err());
            assert!(p.parse_peek("  ").is_err());
            assert!(p.parse_peek(")").is_err());
            assert!(p.parse_peek(" feat").is_err());
            assert!(p.parse_peek(" feat ").is_err());
        }

        #[test]
        fn test_scope() {
            let mut p = scope::<ContextError>;

            // valid
            assert_eq!(p.parse_peek("foo").unwrap(), ("", "foo"));
            assert_eq!(p.parse_peek("Foo").unwrap(), ("", "Foo"));
            assert_eq!(p.parse_peek("FOO").unwrap(), ("", "FOO"));
            assert_eq!(p.parse_peek("fOO").unwrap(), ("", "fOO"));
            assert_eq!(p.parse_peek("foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(p.parse_peek("foo-bar").unwrap(), ("", "foo-bar"));
            assert_eq!(p.parse_peek("x86").unwrap(), ("", "x86"));

            // invalid
            assert!(p.parse_peek("").is_err());
            assert!(p.parse_peek(")").is_err());
        }

        #[test]
        fn test_text() {
            let mut p = text::<ContextError>;

            // valid
            assert_eq!(p.parse_peek("foo").unwrap(), ("", "foo"));
            assert_eq!(p.parse_peek("Foo").unwrap(), ("", "Foo"));
            assert_eq!(p.parse_peek("FOO").unwrap(), ("", "FOO"));
            assert_eq!(p.parse_peek("fOO").unwrap(), ("", "fOO"));
            assert_eq!(p.parse_peek("foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(p.parse_peek("foo bar\n").unwrap(), ("\n", "foo bar"));
            assert_eq!(
                p.parse_peek("foo\nbar\nbaz").unwrap(),
                ("\nbar\nbaz", "foo")
            );

            // invalid
            assert!(p.parse_peek("").is_err());
        }

        #[test]
        fn test_summary() {
            let mut p = summary::<ContextError>;

            // valid
            assert_eq!(
                p.parse_peek("foo: bar").unwrap(),
                ("", ("foo", None, None, "bar"))
            );
            assert_eq!(
                p.parse_peek("foo(bar): baz").unwrap(),
                ("", ("foo", Some("bar"), None, "baz"))
            );
            assert_eq!(
                p.parse_peek("foo(bar):     baz").unwrap(),
                ("", ("foo", Some("bar"), None, "baz"))
            );
            assert_eq!(
                p.parse_peek("foo(bar-baz): qux").unwrap(),
                ("", ("foo", Some("bar-baz"), None, "qux"))
            );
            assert_eq!(
                p.parse_peek("foo!: bar").unwrap(),
                ("", ("foo", None, Some("!"), "bar"))
            );

            // invalid
            assert!(p.parse_peek("").is_err());
            assert!(p.parse_peek(" ").is_err());
            assert!(p.parse_peek("  ").is_err());
            assert!(p.parse_peek("foo").is_err());
            assert!(p.parse_peek("foo bar").is_err());
            assert!(p.parse_peek("foo : bar").is_err());
            assert!(p.parse_peek("foo bar: baz").is_err());
            assert!(p.parse_peek("foo(: bar").is_err());
            assert!(p.parse_peek("foo): bar").is_err());
            assert!(p.parse_peek("foo(): bar").is_err());
            assert!(p.parse_peek("foo(bar)").is_err());
            assert!(p.parse_peek("foo(bar):").is_err());
            assert!(p.parse_peek("foo(bar): ").is_err());
            assert!(p.parse_peek("foo(bar):  ").is_err());
            assert!(p.parse_peek("foo(bar) :baz").is_err());
            assert!(p.parse_peek("foo(bar) : baz").is_err());
            assert!(p.parse_peek("foo (bar): baz").is_err());
            assert!(p.parse_peek("foo bar(baz): qux").is_err());
        }
    }

    mod body {
        use super::*;

        #[test]
        fn test_body() {
            let mut p = body::<ContextError>;

            // valid
            assert_eq!(p.parse_peek("foo").unwrap(), ("", "foo"));
            assert_eq!(p.parse_peek("Foo").unwrap(), ("", "Foo"));
            assert_eq!(p.parse_peek("FOO").unwrap(), ("", "FOO"));
            assert_eq!(p.parse_peek("fOO").unwrap(), ("", "fOO"));
            assert_eq!(
                p.parse_peek("    code block").unwrap(),
                ("", "    code block")
            );
            assert_eq!(p.parse_peek("💃🏽").unwrap(), ("", "💃🏽"));
            assert_eq!(p.parse_peek("foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(
                p.parse_peek("foo\nbar\n\nbaz").unwrap(),
                ("", "foo\nbar\n\nbaz")
            );
            assert_eq!(
                p.parse_peek("foo\n\nBREAKING CHANGE: oops!").unwrap(),
                ("BREAKING CHANGE: oops!", "foo")
            );
            assert_eq!(
                p.parse_peek("foo\n\nBREAKING-CHANGE: bar").unwrap(),
                ("BREAKING-CHANGE: bar", "foo")
            );
            assert_eq!(
                p.parse_peek("foo\n\nMy-Footer: bar").unwrap(),
                ("My-Footer: bar", "foo")
            );
            assert_eq!(
                p.parse_peek("foo\n\nMy-Footer #bar").unwrap(),
                ("My-Footer #bar", "foo")
            );

            // invalid
            assert!(p.parse_peek("").is_err());
        }

        #[test]
        fn test_footer() {
            let mut p = footer::<ContextError>;

            // valid
            assert_eq!(
                p.parse_peek("hello: world").unwrap(),
                ("", ("hello", ":", "world"))
            );
            assert_eq!(
                p.parse_peek("BREAKING CHANGE: woops!").unwrap(),
                ("", ("BREAKING CHANGE", ":", "woops!"))
            );
            assert_eq!(
                p.parse_peek("Co-Authored-By: Marge Simpson <marge@simpsons.com>")
                    .unwrap(),
                (
                    "",
                    ("Co-Authored-By", ":", "Marge Simpson <marge@simpsons.com>")
                )
            );
            assert_eq!(
                p.parse_peek("Closes #12").unwrap(),
                ("", ("Closes", " #", "12"))
            );
            assert_eq!(
                p.parse_peek("BREAKING-CHANGE: broken").unwrap(),
                ("", ("BREAKING-CHANGE", ":", "broken"))
            );

            // invalid
            assert!(p.parse_peek("").is_err());
            assert!(p.parse_peek(" ").is_err());
            assert!(p.parse_peek("  ").is_err());
            assert!(p.parse_peek("foo").is_err());
            assert!(p.parse_peek("foo:").is_err());
            assert!(p.parse_peek("foo: ").is_err());
            assert!(p.parse_peek("foo ").is_err());
            assert!(p.parse_peek("foo #").is_err());
            assert!(p.parse_peek("BREAKING CHANGE").is_err());
            assert!(p.parse_peek("BREAKING CHANGE:").is_err());
            assert!(p.parse_peek("Foo-Bar").is_err());
            assert!(p.parse_peek("Foo-Bar: ").is_err());
            assert!(p.parse_peek("foo").is_err());
        }
    }
}
