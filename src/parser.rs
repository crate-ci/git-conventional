use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_till1, take_while, take_while1};
use nom::character::complete::{char, line_ending};
use nom::character::is_alphabetic;
use nom::combinator::{all_consuming, cut, eof, map, map_parser, opt, peek, verify};
use nom::error::{context, ContextError, ErrorKind, ParseError};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

type CommitDetails<'a> = (
    &'a str,
    Option<&'a str>,
    bool,
    &'a str,
    Option<&'a str>,
    Vec<(&'a str, &'a str, &'a str)>,
);

pub(crate) fn parse<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> Result<CommitDetails<'a>, nom::Err<E>> {
    let (i, subject) = terminated(subject, alt((line_ending, eof)))(i)?;
    let (type_, scope, breaking, description) = subject;

    let (_, body) = opt(tuple((line_ending, body, many0(footer))))(i)?;
    let (body, footers) = body
        .map(|(_, body, footers)| (Some(body), footers))
        .unwrap_or_else(|| (None, Default::default()));

    Ok((type_, scope, breaking.is_some(), description, body, footers))
}

#[inline]
const fn is_line_ending(chr: char) -> bool {
    chr == '\n'
}

/// Accepts any non-empty string slice which starts and ends with an alphabetic
/// character, and has any compound noun character in between.
fn is_compound_noun(s: &str) -> bool {
    for item in s.chars().enumerate() {
        match item {
            (0, chr) if !is_alphabetic(chr as u8) => return false,
            (i, chr) if i + 1 == s.chars().count() && !is_alphabetic(chr as u8) => return false,
            (_, chr) if !is_compound_noun_char(chr) => return false,
            (_, _) => {}
        }
    }

    !s.is_empty()
}

fn is_compound_noun_char(c: char) -> bool {
    is_alphabetic(c as u8) || c == ' ' || c == '-'
}

pub(crate) const BREAKER: &str = "exclamation_mark";

fn exclamation_mark<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(BREAKER, tag("!"))(i)
}

pub(crate) const FORMAT: &str = "format";

fn colon<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(FORMAT, tag(":"))(i)
}

fn space<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(FORMAT, tag(" "))(i)
}

pub(crate) const TYPE: &str = "type";

pub(crate) fn type_<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(TYPE, take_while1(|c: char| is_alphabetic(c as u8)))(i)
}

pub(crate) const SCOPE: &str = "scope";

pub(crate) fn scope<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        SCOPE,
        map_parser(
            take_till1(|c: char| c == ')'),
            all_consuming(verify(take_while(is_compound_noun_char), is_compound_noun)),
        ),
    )(i)
}

pub(crate) const DESCRIPTION: &str = "description";

fn description<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        DESCRIPTION,
        verify(take_till1(is_line_ending), |s: &str| {
            let first = s.chars().next();
            let last = s.chars().last();

            match (first, last) {
                (Some(' '), _) | (_, Some(' ')) => false,
                (_, _) => true,
            }
        }),
    )(i)
}

#[allow(clippy::type_complexity)]
fn subject<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, Option<&'a str>, Option<&'a str>, &'a str), E> {
    tuple((
        type_,
        opt(delimited(char('('), cut(scope), char(')'))),
        opt(exclamation_mark),
        preceded(tuple((colon, space)), description),
    ))(i)
}

pub(crate) const BODY: &str = "body";

fn body<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    if i.is_empty() {
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = E::add_context(i, BODY, err);
        return Err(nom::Err::Error(err));
    }

    let mut offset = 0;
    for line in i.lines() {
        if peek::<_, _, E, _>(tuple((footer_token, footer_separator)))(line).is_ok() {
            offset += 1;
            break;
        }

        offset += line.chars().count() + 1;
    }

    map(take(offset - 1), str::trim_end)(i)
}

fn footer<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, &'a str, &'a str), E> {
    tuple((footer_token, footer_separator, footer_value))(i)
}

pub(crate) fn footer_token<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    alt((
        tag("BREAKING CHANGE"),
        take_while1(|c: char| is_alphabetic(c as u8) || c == '-'),
    ))(i)
}

fn footer_separator<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    alt((tag(": "), tag(" #")))(i)
}

pub(crate) fn footer_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    if i.is_empty() {
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = E::add_context(i, "footer_value", err);
        return Err(nom::Err::Failure(err));
    }

    let mut offset = 0;
    for line in i.lines() {
        if peek::<_, _, E, _>(tuple((footer_token, footer_separator)))(line).is_ok() {
            offset += 1;
            break;
        }

        offset += line.chars().count() + 1;
    }

    map(take(offset - 1), str::trim_end)(i)
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use super::*;
    use nom::error::{convert_error, VerboseError};

    #[allow(clippy::wildcard_enum_match_arm, clippy::print_stdout)]
    fn test<'a, F, O>(f: F, i: &'a str) -> IResult<&'a str, O, VerboseError<&'a str>>
    where
        F: Fn(&'a str) -> IResult<&'a str, O, VerboseError<&'a str>>,
    {
        f(i).map_err(|err| match err {
            nom::Err::Error(err) | nom::Err::Failure(err) => {
                println!("{}", convert_error(i, err.clone()));
                nom::Err::Error(err)
            }
            _ => unreachable!(),
        })
    }

    mod subject {
        use super::*;

        #[test]
        fn test_type() {
            let p = type_::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "Foo").unwrap(), ("", "Foo"));
            assert_eq!(test(p, "FOO").unwrap(), ("", "FOO"));
            assert_eq!(test(p, "fOO").unwrap(), ("", "fOO"));
            assert_eq!(test(p, "foo2bar").unwrap(), ("2bar", "foo"));
            assert_eq!(test(p, "foo-bar").unwrap(), ("-bar", "foo"));
            assert_eq!(test(p, "foo bar").unwrap(), (" bar", "foo"));
            assert_eq!(test(p, "foo: bar").unwrap(), (": bar", "foo"));
            assert_eq!(test(p, "foo(bar").unwrap(), ("(bar", "foo"));
            assert_eq!(test(p, "foo ").unwrap(), (" ", "foo"));

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, ")").is_err());
            assert!(test(p, "@").is_err());
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

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, ")").is_err());
            assert!(test(p, "@").is_err());
            assert!(test(p, "-foo").is_err());
            assert!(test(p, "_foo").is_err());
            assert!(test(p, "foo_bar").is_err());
            assert!(test(p, "foo ").is_err());
            assert!(test(p, "foo2bar").is_err());
        }

        #[test]
        fn test_description() {
            let p = description::<VerboseError<&str>>;

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
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, "foo ").is_err());
            assert!(test(p, " foo").is_err());
            assert!(test(p, " foo ").is_err());
        }

        #[test]
        fn test_subject() {
            let p = subject::<VerboseError<&str>>;

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
                ("", ("hello", ": ", "world"))
            );
            assert_eq!(
                test(p, "BREAKING CHANGE: woops!").unwrap(),
                ("", ("BREAKING CHANGE", ": ", "woops!"))
            );
            assert_eq!(
                test(p, "Co-Authored-By: Marge Simpson <marge@simpsons.com>").unwrap(),
                (
                    "",
                    ("Co-Authored-By", ": ", "Marge Simpson <marge@simpsons.com>")
                )
            );
            assert_eq!(test(p, "Closes #12").unwrap(), ("", ("Closes", " #", "12")));
            assert_eq!(
                test(p, "BREAKING-CHANGE: broken").unwrap(),
                ("", ("BREAKING-CHANGE", ": ", "broken"))
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
