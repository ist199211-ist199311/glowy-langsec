use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    multi::{many_m_n, separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated},
};

use crate::{
    ast::ConstDeclSpecNode, eol, exprs::expression, identifier, msp00, msp01, NResult, Span,
};

fn const_spec(s: Span) -> NResult<ConstDeclSpecNode> {
    // TODO: support type
    let (s, ids) = separated_list1(char(','), msp00(identifier))(s)?;
    let (s, exprs) = preceded(
        terminated(char('='), multispace0),
        separated_list1(msp00(char(',')), expression),
    )(s)?;

    if let Ok(node) = ConstDeclSpecNode::try_new(ids, exprs) {
        Ok((s, node))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Fail, // hack!
        )))
    }
}

pub fn const_decl(s: Span) -> NResult<Vec<ConstDeclSpecNode>> {
    delimited(
        msp01(tag("const")),
        alt((
            map(const_spec, |spec| vec![spec]),
            delimited(
                char('('),
                // vv why is this not list1... I thought Google had vowed not to be evil...
                separated_list0(eol, const_spec),
                terminated(
                    many_m_n(0, 1, msp00(char(';'))),
                    preceded(multispace0, char(')')),
                ),
            ),
        )),
        eol,
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{ExprNode, LiteralNode, OperandNameNode},
        tests::{assert_eq_result, span},
    };

    #[test]
    fn simple() {
        assert_eq_result(
            vec![ConstDeclSpecNode {
                mapping: vec![(span("a", 6, 1), ExprNode::Literal(LiteralNode::Int(3)))],
            }],
            const_decl(Span::new("const a = 3")),
        )
    }

    #[test]
    fn names() {
        assert_eq_result(
            vec![ConstDeclSpecNode {
                mapping: vec![
                    (
                        span("a", 6, 1),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: span("x", 16, 1),
                        }),
                    ),
                    (
                        span("b", 9, 1),
                        ExprNode::Name(OperandNameNode {
                            package: Some(span("pkg", 19, 1)),
                            id: span("y", 24, 1),
                        }),
                    ),
                    (span("c", 12, 1), ExprNode::Literal(LiteralNode::Int(451))),
                ],
            }],
            const_decl(Span::new("const a, b, c = x, pkg. y, 451")),
        )
    }

    #[test]
    fn multi() {
        assert_eq_result(
            vec![
                ConstDeclSpecNode {
                    mapping: vec![(span("X3", 9, 2), ExprNode::Literal(LiteralNode::Int(4)))],
                },
                ConstDeclSpecNode {
                    mapping: vec![
                        (span("m", 17, 3), ExprNode::Literal(LiteralNode::Int(9))),
                        (span("n", 20, 3), ExprNode::Literal(LiteralNode::Int(6))),
                    ],
                },
            ],
            const_decl(Span::new("const\t\n (X3 = 4\n m, n\t= 9, 6;)")),
        );
    }
}
