use ast::{PackageClauseNode, SourceFileNode, TopLevelDeclNode};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, line_ending, multispace0, multispace1, space0},
    combinator::{eof, recognize},
    error::ParseError,
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded},
    AsChar, IResult, InputTakeAtPosition, Parser,
};
use nom_locate::LocatedSpan;

pub mod ast;
mod consts;

pub type Span<'a> = LocatedSpan<&'a str>;
type SResult<'a> = IResult<Span<'a>, Span<'a>>;
type NResult<'a, T> = IResult<Span<'a>, T>;

fn msp00<'a, T: 'a, F, O, E>(inner: F) -> impl Parser<T, O, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Parser<T, O, E>,
    E: ParseError<T>,
{
    delimited(multispace0, inner, multispace0)
}

fn msp01<'a, T: 'a, F, O, E>(inner: F) -> impl Parser<T, O, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Parser<T, O, E>,
    E: ParseError<T>,
{
    delimited(multispace0, inner, multispace1)
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
    alt((msp00(tag(";")), preceded(space0, alt((line_ending, eof)))))(s)
}

fn package(s: Span) -> NResult<PackageClauseNode> {
    let (s, id) = delimited(msp01(tag("package")), identifier, eol)(s)?;

    Ok((s, PackageClauseNode { id }))
}

fn top_level_decls(s: Span) -> NResult<Vec<TopLevelDeclNode>> {
    fn top_const(s: Span) -> NResult<TopLevelDeclNode> {
        let (s, specs) = consts::const_decl(s)?;
        Ok((s, TopLevelDeclNode::Const(specs)))
    }

    many0(alt((top_const /* TODO: more... */,)))(s)
}

fn source_file(s: Span) -> NResult<SourceFileNode> {
    let (s, package_clause) = msp00(package).parse(s)?;
    let (s, top_level_decls) = top_level_decls(s)?;

    Ok((
        s,
        SourceFileNode {
            package_clause,
            top_level_decls,
        },
    ))
}

pub fn parse(input: &str) -> Result<SourceFileNode, Option<Span>> {
    match source_file(Span::new(input)) {
        Ok((_, node)) => Ok(node),
        Err(nom::Err::Error(inner)) => Err(Some(inner.input)),
        _ => Err(None),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    use crate::ast::ConstDeclSpecNode;

    pub fn span(fragment: &str, offset: usize, line: u32) -> Span {
        unsafe { Span::new_from_raw_offset(offset, line, fragment, ()) }
    }

    pub fn assert_eq_result<T>(expected: T, result: IResult<Span, T>)
    where
        T: Debug + PartialEq,
    {
        if let Ok((s, real)) = result {
            if s.is_empty() {
                assert_eq!(expected, real);
            } else {
                panic!("{s} is not an empty Span!")
            }
        } else {
            panic!("parser failed: {result:?}")
        }
    }

    #[test]
    fn package_no_semi_colon() {
        assert_eq!(
            Ok(SourceFileNode {
                package_clause: PackageClauseNode {
                    id: span("ab12_3F", 17, 3),
                },
                top_level_decls: vec![]
            }),
            parse("\n\n    \tpackage   ab12_3F\t\n\t")
        );
    }

    #[test]
    fn package_semi_colon() {
        assert_eq!(
            Ok(SourceFileNode {
                package_clause: PackageClauseNode {
                    id: span("ABC", 8, 1),
                },
                top_level_decls: vec![]
            }),
            parse("package ABC\t  ;\n")
        );
    }

    #[test]
    fn multiple_const_decls() {
        assert_eq!(
            Ok(SourceFileNode {
                package_clause: PackageClauseNode {
                    id: span("name", 8, 1),
                },
                top_level_decls: vec![
                    TopLevelDeclNode::Const(vec![ConstDeclSpecNode {
                        ids: vec![span("abc3", 20, 3)]
                    }]),
                    TopLevelDeclNode::Const(vec![
                        ConstDeclSpecNode {
                            ids: vec![span("a", 38, 5),]
                        },
                        ConstDeclSpecNode {
                            ids: vec![span("b", 46, 6), span("c", 50, 7)]
                        }
                    ])
                ]
            }),
            parse(concat!(
                "package name\n\n",
                "const abc3 = 42\n",
                "const \n(a =\n 7; b\n, c = 8, 9);"
            ))
        );
    }
}
