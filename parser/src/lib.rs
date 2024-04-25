use ast::{Node, NodeKind};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, line_ending, multispace0, space0, space1},
    combinator::{eof, recognize},
    error::ParseError,
    multi::many0_count,
    sequence::{delimited, pair, preceded},
    AsChar, IResult, InputTakeAtPosition, Parser,
};
use nom_locate::LocatedSpan;

pub mod ast;

type Span<'a> = LocatedSpan<&'a str>;
type SResult<'a> = IResult<Span<'a>, Span<'a>>;
type NResult<'a> = IResult<Span<'a>, Node<'a>>;

fn sp01<'a, T: 'a, F, O, E>(inner: F) -> impl Parser<T, O, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Parser<T, O, E>,
    E: ParseError<T>,
{
    delimited(space0, inner, space1)
}

fn msp<'a, T: 'a, F, O, E>(inner: F) -> impl Parser<T, O, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Parser<T, O, E>,
    E: ParseError<T>,
{
    delimited(multispace0, inner, multispace0)
}

fn unicode_digit(s: Span) -> SResult {
    // FIXME: support non-ASCII
    digit1(s)
}

fn letter(s: Span) -> SResult {
    alt((alpha1, tag("_")))(s)
}

fn identifier(s: Span) -> SResult {
    recognize(pair(letter, many0_count(alt((letter, unicode_digit)))))(s)
}

fn eol(s: Span) -> SResult {
    // not actually necessarily end of line, can also just be a semi-colon
    alt((
        delimited(space0, tag(";"), multispace0),
        preceded(space0, alt((line_ending, eof))),
    ))(s)
}

fn package(s: Span) -> NResult {
    let (s, id) = delimited(sp01(tag("package")), identifier, eol)(s)?;

    Ok((s, Node::new(NodeKind::PackageClause { id })))
}

fn source_file(s: Span) -> NResult {
    msp(package).parse(s)
}

pub fn parse(input: &str) -> Result<Node, Option<Span>> {
    match source_file(Span::new(input)) {
        Ok((_, node)) => Ok(node),
        Err(nom::Err::Error(inner)) => Err(Some(inner.input)),
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(fragment: &str, offset: usize, line: u32) -> Span {
        unsafe { Span::new_from_raw_offset(offset, line, fragment, ()) }
    }

    #[test]
    fn package_no_semi_colon() {
        assert_eq!(
            Ok(Node::new(NodeKind::PackageClause {
                id: span("ab12_3F", 17, 3)
            })),
            parse("\n\n    \tpackage   ab12_3F\t\n\t")
        );
    }

    #[test]
    fn package_semi_colon() {
        assert_eq!(
            Ok(Node::new(NodeKind::PackageClause {
                id: span("ABC", 8, 1)
            })),
            parse("package ABC\t  ;\n")
        );
    }
}
