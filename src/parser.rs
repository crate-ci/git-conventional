use nom::bytes::complete::{tag, take, take_till1, take_while, take_while1};
use nom::character::complete::{char, line_ending, space0};
use nom::character::is_alphanumeric;
use nom::combinator::{all_consuming, cut, map_parser, opt, verify};
use nom::error::{context, ErrorKind, ParseError};
use nom::sequence::{delimited, preceded, tuple};
use nom::{FindSubstring, IResult, InputTake};
use std::str;

type CommitDetails<'a> = (
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
);

pub(crate) fn parse<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> Result<CommitDetails<'a>, nom::Err<E>> {
    let (i, (type_, scope, breaking, description)) = header(i)?;
    let (i, body) = body(i)?;
    let (_, breaking_change) = breaking_change(i)?;

    Ok((type_, scope, breaking, description, body, breaking_change))
}

#[inline]
const fn is_line_ending(chr: char) -> bool {
    chr == '\n'
}

/// Accepts any non-empty string slice which starts and ends with an
/// alphanumeric character, and has any compound noun character in between.
fn is_compound_noun(s: &str) -> bool {
    for item in s.chars().enumerate() {
        match item {
            (0, chr) if !is_alphanumeric(chr as u8) => return false,
            (i, chr) if i + 1 == s.chars().count() && !is_alphanumeric(chr as u8) => return false,
            (_, chr) if !is_compound_noun_char(chr) => return false,
            (_, _) => {}
        }
    }

    !s.is_empty()
}

fn is_compound_noun_char(c: char) -> bool {
    is_alphanumeric(c as u8) || c == ' ' || c == '-'
}

fn not_blank_line<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    match i.find("\n\n") {
        Some(index) => Ok(i.take_split(index)),
        None => Ok(("", i)),
    }
}

fn exclamation_mark<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context("exclamation_mark", tag("!"))(i)
}

fn colon<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context("colon", tag(":"))(i)
}

fn space<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context("space", tag(" "))(i)
}

fn type_<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "type",
        verify(take_while1(is_compound_noun_char), is_compound_noun),
    )(i)
}

fn scope<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "scope",
        map_parser(
            take_till1(|c: char| c == ')'),
            all_consuming(verify(take_while(is_compound_noun_char), is_compound_noun)),
        ),
    )(i)
}

fn description<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context("description", preceded(space0, take_till1(is_line_ending)))(i)
}

fn header<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, Option<&'a str>, Option<&'a str>, &'a str), E> {
    tuple((
        type_,
        opt(delimited(char('('), cut(scope), char(')'))),
        opt(exclamation_mark),
        preceded(tuple((colon, space)), description),
    ))(i)
}

fn body<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Option<&'a str>, E> {
    if i.is_empty() {
        return Ok(("", None));
    }

    let (i, _) = tuple((line_ending, line_ending))(i)?;
    if i.is_empty() {
        let err = E::from_error_kind(i, ErrorKind::Eof);
        let err = E::add_context(i, "body", err);
        return Err(nom::Err::Error(err));
    }

    let end = i
        .find_substring("BREAKING CHANGE: ")
        .unwrap_or_else(|| i.chars().count());

    take(end)(i).map(|(i, out)| (i, Some(out)))
}

fn breaking_change<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Option<&'a str>, E> {
    opt(preceded(tag("BREAKING CHANGE: "), not_blank_line))(i)
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used, clippy::non_ascii_literal)]
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

    mod header {
        use super::*;

        #[test]
        fn test_type() {
            let p = type_::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo2bar").unwrap(), ("", "foo2bar"));
            assert_eq!(test(p, "foo-bar").unwrap(), ("", "foo-bar"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo(bar").unwrap(), ("(bar", "foo"));

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
            assert!(test(p, ")").is_err());
            assert!(test(p, "@").is_err());
            assert!(test(p, " feat").is_err());
            assert!(test(p, "feat ").is_err());
            assert!(test(p, " feat ").is_err());
        }

        #[test]
        fn test_scope() {
            let p = scope::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo2bar").unwrap(), ("", "foo2bar"));
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
        }

        #[test]
        fn test_description() {
            let p = description::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "foo").unwrap(), ("", "foo"));
            assert_eq!(test(p, "foo bar").unwrap(), ("", "foo bar"));
            assert_eq!(test(p, "foo bar\n").unwrap(), ("\n", "foo bar"));
            assert_eq!(test(p, "foo\nbar\nbaz").unwrap(), ("\nbar\nbaz", "foo"));
            assert_eq!(test(p, "  foo").unwrap(), ("", "foo"));

            // invalid
            assert!(test(p, "").is_err());
            assert!(test(p, " ").is_err());
            assert!(test(p, "  ").is_err());
        }

        #[test]
        fn test_header() {
            let p = header::<VerboseError<&str>>;

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
            assert!(test(p, "foo(: bar").is_err());
            assert!(test(p, "foo): bar").is_err());
            assert!(test(p, "foo(): bar").is_err());
            assert!(test(p, "foo(bar)").is_err());
            assert!(test(p, "foo(bar):").is_err());
            assert!(test(p, "foo(bar): ").is_err());
            assert!(test(p, "foo(bar) :baz").is_err());
            assert!(test(p, "foo(bar) : baz").is_err());
            // assert!(test(p, "foo bar(baz): qux").is_err());
            // assert!(test(p, "foo(bar baz): qux").is_err());
        }
    }

    mod body {
        use super::*;

        #[test]
        fn test_body() {
            let p = body::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "").unwrap(), ("", None));
            assert_eq!(
                test(p, "\n\n    code block").unwrap(),
                ("", Some("    code block"))
            );
            assert_eq!(test(p, "\n\n💃🏽").unwrap(), ("", Some("💃🏽")));
            assert_eq!(test(p, "\n\nfoo bar").unwrap(), ("", Some("foo bar")));
            assert_eq!(
                test(p, "\n\nfoo\nbar\n\nbaz").unwrap(),
                ("", Some("foo\nbar\n\nbaz"))
            );
            assert_eq!(
                test(p, "\n\nfoo\n\nBREAKING CHANGE: oops!").unwrap(),
                ("BREAKING CHANGE: oops!", Some("foo\n\n"))
            );

            // invalid
            assert!(test(p, " ").is_err());
            assert!(test(p, "foo").is_err());
            assert!(test(p, "\n").is_err());
            assert!(test(p, "\n ").is_err());
            assert!(test(p, "\nfoo").is_err());
            assert!(test(p, "\n\n").is_err());
        }

        #[test]
        #[rustfmt::skip]
        fn test_breaking_change() {
            let p = breaking_change::<VerboseError<&str>>;

            // valid
            assert_eq!(test(p, "").unwrap(), ("", None));
            assert_eq!(test(p, " ").unwrap(), (" ", None));
            assert_eq!(test(p, "  ").unwrap(), ("  ", None));
            assert_eq!(test(p, "foo").unwrap(), ("foo", None));
            assert_eq!(test(p, "BREAKING CHANGE").unwrap(), ("BREAKING CHANGE", None));
            assert_eq!(test(p, "BREAKING CHANGE:").unwrap(), ("BREAKING CHANGE:", None));
            assert_eq!(test(p, "BREAKING CHANGE: ").unwrap(), ("", Some("")));
            assert_eq!(test(p, "BREAKING CHANGE: foo bar").unwrap(), ("", Some("foo bar")));
            assert_eq!(test(p, "BREAKING CHANGE: foo\nbar").unwrap(), ("", Some("foo\nbar")));
            assert_eq!(test(p, "BREAKING CHANGE: 1\n2\n\n3").unwrap(), ("\n\n3", Some("1\n2")));
        }
    }
}
