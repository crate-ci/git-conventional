use std::str;

use winnow::branch::alt;
use winnow::bytes::complete::{tag, take, take_till1, take_while, take_while1};
use winnow::character::complete::{char, line_ending};
use winnow::combinator::{cut, eof, fail, map, opt, peek};
use winnow::error::{context, ContextError, ErrorKind, ParseError};
use winnow::multi::many0;
use winnow::multi::many0_count;
use winnow::sequence::{delimited, preceded, terminated, tuple};
use winnow::IResult;
use winnow::Parser;

type CommitDetails<'a> = (
    &'a str,
    Option<&'a str>,
    bool,
    &'a str,
    Option<&'a str>,
    Vec<(&'a str, &'a str, &'a str)>,
);

pub(crate) fn parse<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> Result<CommitDetails<'a>, winnow::Err<E>> {
    let (_i, c) = trace("message", message)(i)?;
    debug_assert!(_i.is_empty(), "{:?} remaining", _i);
    Ok(c)
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

fn whitespace<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    take_while(is_whitespace)(i)
}

// <message>         ::= <summary>, <newline>+, <body>, (<newline>+, <footer>)*
//                    |  <summary>, (<newline>+, <footer>)*
//                    |  <summary>, <newline>*
pub(crate) fn message<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, CommitDetails<'a>, E> {
    let (i, summary) = terminated(trace("summary", summary), alt((line_ending, eof)))(i)?;
    let (type_, scope, breaking, description) = summary;

    // The body MUST begin one blank line after the description.
    let (i, _) = context(BODY, alt((line_ending, eof)))(i)?;

    let (i, _extra): (_, ()) = many0(line_ending)(i)?;

    let (i, body) = opt(trace("body", body))(i)?;

    let (i, footers) = many0(trace("footer", footer))(i)?;

    let (i, _) = many0_count(line_ending)(i)?;

    Ok((
        i,
        (type_, scope, breaking.is_some(), description, body, footers),
    ))
}

// <type>            ::= <any UTF8-octets except newline or parens or ":" or "!:" or whitespace>+
pub(crate) fn type_<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        TYPE,
        take_while1(|c: char| {
            !is_line_ending(c) && !is_parens(c) && c != ':' && c != '!' && !is_whitespace(c)
        }),
    )(i)
}

pub(crate) const TYPE: &str = "type";

// <scope>           ::= <any UTF8-octets except newline or parens>+
pub(crate) fn scope<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        SCOPE,
        take_while1(|c: char| !is_line_ending(c) && !is_parens(c)),
    )(i)
}

pub(crate) const SCOPE: &str = "scope";

// /* "!" should be added to the AST as a <breaking-change> node with the value "!" */
// <summary>         ::= <type>, "(", <scope>, ")", ["!"], ":", <whitespace>*, <text>
//                    |  <type>, ["!"], ":", <whitespace>*, <text>
#[allow(clippy::type_complexity)]
fn summary<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, Option<&'a str>, Option<&'a str>, &'a str), E> {
    context(
        SUMMARY,
        tuple((
            trace("type", type_),
            opt(delimited(char('('), cut(trace("scope", scope)), char(')'))),
            opt(exclamation_mark),
            preceded(
                tuple((tag(":"), whitespace)),
                context(DESCRIPTION, trace("description", text)),
            ),
        )),
    )(i)
}

pub(crate) const SUMMARY: &str = "SUMMARY";
pub(crate) const DESCRIPTION: &str = "description";

// <text>            ::= <any UTF8-octets except newline>*
fn text<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    take_till1(is_line_ending)(i)
}

fn body<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    if i.is_empty() {
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = err.add_context(i, BODY);
        return Err(winnow::Err::Backtrack(err));
    }

    let mut offset = 0;
    let mut prior_is_empty = true;
    for line in crate::lines::LinesWithTerminator::new(i) {
        if prior_is_empty && peek::<_, _, E, _>(tuple((token, separator)))(line.trim_end()).is_ok()
        {
            break;
        }
        prior_is_empty = line.trim().is_empty();

        offset += line.chars().count();
    }
    if offset == 0 {
        fail::<_, (), _>(i)?;
    }

    map(take(offset), str::trim_end)(i)
}

pub(crate) const BODY: &str = "body";

// <footer>          ::= <token>, <separator>, <whitespace>*, <value>
fn footer<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, &'a str, &'a str), E> {
    tuple((token, separator, whitespace, value))
        .map(|(ft, s, _, fv)| (ft, s, fv))
        .parse(i)
}

// <token>           ::= <breaking-change>
//                    |  <type>
pub(crate) fn token<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    alt((tag("BREAKING CHANGE"), type_))(i)
}

// <separator>       ::= ":" | " #"
fn separator<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    alt((tag(":"), tag(" #")))(i)
}

pub(crate) fn value<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    if i.is_empty() {
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = err.add_context(i, "value");
        return Err(winnow::Err::Cut(err));
    }

    let mut offset = 0;
    for (i, line) in crate::lines::LinesWithTerminator::new(i).enumerate() {
        if 0 < i && peek::<_, _, E, _>(tuple((token, separator)))(line.trim_end()).is_ok() {
            break;
        }

        offset += line.chars().count();
    }

    map(take(offset), str::trim_end)(i)
}

fn exclamation_mark<'a, E: ParseError<&'a str> + ContextError<&'a str> + std::fmt::Debug>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(BREAKER, tag("!"))(i)
}

pub(crate) const BREAKER: &str = "exclamation_mark";

#[cfg(feature = "unstable-trace")]
pub(crate) fn trace<I: std::fmt::Debug, O: std::fmt::Debug, E: std::fmt::Debug>(
    context: impl std::fmt::Display,
    mut parser: impl winnow::Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, O, E> {
    static DEPTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    move |input: I| {
        let depth = DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst) * 2;
        eprintln!("{:depth$}--> {} {:?}", "", context, input);
        match parser.parse(input) {
            Ok((i, o)) => {
                DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                eprintln!("{:depth$}<-- {} {:?}", "", context, i);
                Ok((i, o))
            }
            Err(err) => {
                DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                eprintln!("{:depth$}<-- {} {:?}", "", context, err);
                Err(err)
            }
        }
    }
}

#[cfg(not(feature = "unstable-trace"))]
pub(crate) fn trace<I: std::fmt::Debug, O: std::fmt::Debug, E: std::fmt::Debug>(
    _context: impl std::fmt::Display,
    mut parser: impl winnow::Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |input: I| parser.parse(input)
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use super::*;
    use winnow::error::{convert_error, VerboseError};

    #[allow(clippy::wildcard_enum_match_arm, clippy::print_stdout)]
    fn test<'a, F, O>(f: F, i: &'a str) -> IResult<&'a str, O, VerboseError<&'a str>>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, VerboseError<&'a str>>,
    {
        f(i).map_err(|err| match err {
            winnow::Err::Backtrack(err) | winnow::Err::Cut(err) => {
                println!("{}", convert_error(i, err.clone()));
                winnow::Err::Backtrack(err)
            }
            _ => unreachable!(),
        })
    }

    mod message {
        use super::*;
        #[test]
        fn errors() {
            let p = message::<VerboseError<&str>>;

            let input = "Hello World";
            let err = test(p, input).unwrap_err();
            let err = crate::Error::with_nom(input, err);
            assert_eq!(err.to_string(), crate::ErrorKind::MissingType.to_string());

            let input = "fix Improved error messages\n";
            let err = test(p, input).unwrap_err();
            let err = crate::Error::with_nom(input, err);
            assert_eq!(err.to_string(), crate::ErrorKind::MissingType.to_string());
        }
    }

    mod summary {
        use super::*;

        #[test]
        fn test_type() {
            let p = type_::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "Foo").unwrap(), ("", "Foo"));
            assert_eq!(test(p, "FOO").unwrap(), ("", "FOO"));
            assert_eq!(test(p, "fOO").unwrap(), ("", "fOO"));
            assert_eq!(test(p, "foo2bar").unwrap(), ("", "foo2bar"));
            assert_eq!(test(p, "foo-bar").unwrap(), ("", "foo-bar"));
            assert_eq!(test(p, "foo bar").unwrap(), (" bar", "foo"));
            assert_eq!(test(p, "foo: bar").unwrap(), (": bar", "foo"));
            assert_eq!(test(p, "foo!: bar").unwrap(), ("!: bar", "foo"));
            assert_eq!(test(p, "foo(bar").unwrap(), ("(bar", "foo"));
            assert_eq!(test(p, "foo ").unwrap(), (" ", "foo"));

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, ")").is_err());
            assert!(test(p, " feat").is_err());
            assert!(test(p, " feat ").is_err());
        }

        #[test]
        fn test_scope() {
            let p = scope::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "Foo").unwrap(), ("", "Foo"));
            assert_eq!(test(p, "FOO").unwrap(), ("", "FOO"));
            assert_eq!(test(p, "fOO").unwrap(), ("", "fOO"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo-bar").unwrap(), ("", "foo-bar"));
            assert_eq!(test(p, "x86").unwrap(), ("", "x86"));

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, ")").is_err());
        }

        #[test]
        fn test_text() {
            let p = text::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "Foo").unwrap(), ("", "Foo"));
            assert_eq!(test(p, "FOO").unwrap(), ("", "FOO"));
            assert_eq!(test(p, "fOO").unwrap(), ("", "fOO"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo bar\n").unwrap(), ("\n", "foo bar"));
            assert_eq!(test(p, "foo\nbar\nbaz").unwrap(), ("\nbar\nbaz", "foo"));

            // invalid
            assert!(test(p, "").is_err());
        }

        #[test]
        fn test_summary() {
            let p = summary::<VerboseError<&str>>;

            // valid
            assert_eq!(
                test(p, "foo: bar").unwrap(),
                ("", ("foo", None, None, "bar"))
            );
            assert_eq!(
                test(p, "foo(bar): baz").unwrap(),
                ("", ("foo", Some("bar"), None, "baz"))
            );
            assert_eq!(
                test(p, "foo(bar):     baz").unwrap(),
                ("", ("foo", Some("bar"), None, "baz"))
            );
            assert_eq!(
                test(p, "foo(bar-baz): qux").unwrap(),
                ("", ("foo", Some("bar-baz"), None, "qux"))
            );
            assert_eq!(
                test(p, "foo!: bar").unwrap(),
                ("", ("foo", None, Some("!"), "bar"))
            );

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, "foo").is_err());
            assert!(test(p, "foo bar").is_err());
            assert!(test(p, "foo : bar").is_err());
            assert!(test(p, "foo bar: baz").is_err());
            assert!(test(p, "foo(: bar").is_err());
            assert!(test(p, "foo): bar").is_err());
            assert!(test(p, "foo(): bar").is_err());
            assert!(test(p, "foo(bar)").is_err());
            assert!(test(p, "foo(bar):").is_err());
            assert!(test(p, "foo(bar): ").is_err());
            assert!(test(p, "foo(bar):  ").is_err());
            assert!(test(p, "foo(bar) :baz").is_err());
            assert!(test(p, "foo(bar) : baz").is_err());
            assert!(test(p, "foo (bar): baz").is_err());
            assert!(test(p, "foo bar(baz): qux").is_err());
        }
    }

    mod body {
        use super::*;

        #[test]
        fn test_body() {
            let p = body::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "Foo").unwrap(), ("", "Foo"));
            assert_eq!(test(p, "FOO").unwrap(), ("", "FOO"));
            assert_eq!(test(p, "fOO").unwrap(), ("", "fOO"));
            assert_eq!(test(p, "    code block").unwrap(), ("", "    code block"));
            assert_eq!(test(p, "💃🏽").unwrap(), ("", "💃🏽"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo\nbar\n\nbaz").unwrap(), ("", "foo\nbar\n\nbaz"));
            assert_eq!(
                test(p, "foo\n\nBREAKING CHANGE: oops!").unwrap(),
                ("BREAKING CHANGE: oops!", "foo")
            );
            assert_eq!(
                test(p, "foo\n\nBREAKING-CHANGE: bar").unwrap(),
                ("BREAKING-CHANGE: bar", "foo")
            );
            assert_eq!(
                test(p, "foo\n\nMy-Footer: bar").unwrap(),
                ("My-Footer: bar", "foo")
            );
            assert_eq!(
                test(p, "foo\n\nMy-Footer #bar").unwrap(),
                ("My-Footer #bar", "foo")
            );

            // invalid
            assert!(test(p, "").is_err());
        }

        #[test]
        fn test_footer() {
            let p = footer::<VerboseError<&str>>;

            // valid
            assert_eq!(
                test(p, "hello: world").unwrap(),
                ("", ("hello", ":", "world"))
            );
            assert_eq!(
                test(p, "BREAKING CHANGE: woops!").unwrap(),
                ("", ("BREAKING CHANGE", ":", "woops!"))
            );
            assert_eq!(
                test(p, "Co-Authored-By: Marge Simpson <marge@simpsons.com>").unwrap(),
                (
                    "",
                    ("Co-Authored-By", ":", "Marge Simpson <marge@simpsons.com>")
                )
            );
            assert_eq!(test(p, "Closes #12").unwrap(), ("", ("Closes", " #", "12")));
            assert_eq!(
                test(p, "BREAKING-CHANGE: broken").unwrap(),
                ("", ("BREAKING-CHANGE", ":", "broken"))
            );

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, "foo").is_err());
            assert!(test(p, "foo:").is_err());
            assert!(test(p, "foo: ").is_err());
            assert!(test(p, "foo ").is_err());
            assert!(test(p, "foo #").is_err());
            assert!(test(p, "BREAKING CHANGE").is_err());
            assert!(test(p, "BREAKING CHANGE:").is_err());
            assert!(test(p, "Foo-Bar").is_err());
            assert!(test(p, "Foo-Bar: ").is_err());
            assert!(test(p, "foo").is_err());
        }
    }
}
