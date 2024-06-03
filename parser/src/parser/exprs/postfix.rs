use super::{parse_expression, parse_expressions_list_while};
use crate::{
    ast::{CallNode, ExprNode, IndexingNode},
    parser::{expect, of_kind, types::parse_channel_type, PResult},
    token::{Token, TokenKind},
    TokenStream,
};

pub fn parse_call<'a>(s: &mut TokenStream<'a>, func: ExprNode<'a>) -> PResult<'a, CallNode<'a>> {
    let paren = expect(s, TokenKind::ParenL, Some("function call"))?;

    // TODO: support trailing comma

    // TODO: support type arguments besides (non-receive) channel types
    // ^ (in general, indistinguishable from expression? e.g. "int" vs "abc")

    // cannot support receive channel types: f(<-some_channel) or f(<-chan int) ??
    let type_arg = if let Some(Ok(of_kind!(TokenKind::Chan))) = s.peek() {
        let channel_type = parse_channel_type(s)?;

        // if this was not the only argument
        if let Some(Ok(of_kind!(TokenKind::Comma))) = s.peek() {
            s.next(); // continue to actual arguments
        }

        Some(channel_type)
    } else {
        None
    };

    let args = parse_expressions_list_while(s, |token| {
        !matches!(token.kind, TokenKind::Ellipsis | TokenKind::ParenR)
    })?
    .unwrap_or_else(Vec::new); // got end-of-file, but it's fine because the upcoming expect will fail

    let variadic = if let Some(Ok(of_kind!(TokenKind::Ellipsis))) = s.peek() {
        s.next(); // advance

        true
    } else {
        false
    };

    expect(s, TokenKind::ParenR, Some("function call"))?;

    Ok(CallNode {
        func: Box::new(func),
        type_arg,
        args,
        variadic,
        location: s.location_since(&paren),
        annotation: paren.annotation,
    })
}

pub fn parse_indexing<'a>(
    s: &mut TokenStream<'a>,
    expr: ExprNode<'a>,
) -> PResult<'a, IndexingNode<'a>> {
    let open = expect(s, TokenKind::SquareL, Some("indexing expression"))?;

    let index = parse_expression(s)?;

    // optional trailing comma
    if let Some(Ok(of_kind!(TokenKind::Comma))) = s.peek() {
        s.next(); // advance
    }

    expect(s, TokenKind::SquareR, Some("indexing expression"))?;

    Ok(IndexingNode {
        expr: Box::new(expr),
        index: Box::new(index),
        location: s.location_since(&open),
    })
}

pub fn parse_postfix_if_exists<'a>(
    s: &mut TokenStream<'a>,
    operand: ExprNode<'a>,
) -> PResult<'a, ExprNode<'a>> {
    let expr = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::ParenL)) => parse_call(s, operand)?.into(),
        Some(of_kind!(TokenKind::SquareL)) => parse_indexing(s, operand)?.into(),
        _ => return Ok(operand), // nothing found, stop the recursion
    };

    parse_postfix_if_exists(s, expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{
            BinaryOpKind, ChannelDirection, LiteralNode, OperandNameNode, TypeNode, UnaryOpKind,
        },
        lexer::Lexer,
        parser::exprs::parse_expression,
        Span,
    };

    fn parse(input: &str) -> PResult<'_, ExprNode<'_>> {
        let mut stream = TokenStream::new(Lexer::new(input));

        parse_expression(&mut stream)
    }

    #[test]
    fn call() {
        assert_eq!(
            ExprNode::Call(CallNode {
                func: Box::new(ExprNode::Call(CallNode {
                    func: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Sum,
                        left: Box::new(ExprNode::Name(OperandNameNode {
                            package: Some(Span::new("abc", 1, 1)),
                            id: Span::new("def", 5, 1)
                        })),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(14))),
                        location: 1..13,
                    }),
                    type_arg: None,
                    args: vec![
                        ExprNode::BinaryOp {
                            kind: BinaryOpKind::Sum,
                            left: Box::new(ExprNode::Literal(LiteralNode::Int(21))),
                            right: Box::new(ExprNode::BinaryOp {
                                kind: BinaryOpKind::Product,
                                left: Box::new(ExprNode::Literal(LiteralNode::Int(7))),
                                right: Box::new(ExprNode::UnaryOp {
                                    kind: UnaryOpKind::Negation,
                                    operand: Box::new(ExprNode::Literal(LiteralNode::Int(9))),
                                    location: 24..26,
                                }),
                                location: 20..26,
                            }),
                            location: 15..26,
                        },
                        ExprNode::Literal(LiteralNode::Int(0))
                    ],
                    variadic: true,
                    location: 14..33,
                    annotation: None,
                })),
                type_arg: None,
                args: vec![],
                variadic: false,
                location: 33..35,
                annotation: None
            }),
            parse("(abc.def + 14)(21 + 7 * -9, 0...)()").unwrap()
        )
    }

    #[test]
    fn call_index() {
        assert_eq!(
            ExprNode::Call(CallNode {
                func: Box::new(ExprNode::Indexing(IndexingNode {
                    expr: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Sum,
                        left: Box::new(ExprNode::Name(OperandNameNode {
                            package: Some(Span::new("abc", 1, 1)),
                            id: Span::new("def", 5, 1)
                        })),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(14))),
                        location: 1..13,
                    }),
                    index: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Sum,
                        left: Box::new(ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("k", 15, 1)
                        })),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(2))),
                        location: 15..20,
                    }),
                    location: 14..22,
                })),
                type_arg: None,
                args: vec![],
                variadic: false,
                location: 22..24,
                annotation: None
            }),
            parse("(abc.def + 14)[k + 2,]()").unwrap()
        )
    }

    #[test]
    fn call_with_type_arg() {
        assert_eq!(
            ExprNode::Call(CallNode {
                func: Box::new(ExprNode::Name(OperandNameNode {
                    package: Some(Span::new("p", 0, 1)),
                    id: Span::new("f", 2, 1)
                })),
                type_arg: Some(TypeNode::Channel {
                    r#type: Box::new(TypeNode::Name {
                        package: None,
                        id: Span::new("int", 9, 1),
                        args: vec![]
                    }),
                    direction: None,
                }),
                args: vec![
                    ExprNode::Literal(LiteralNode::Rune('\u{0007}')),
                    ExprNode::Call(CallNode {
                        func: Box::new(ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("g", 20, 1)
                        })),
                        type_arg: Some(TypeNode::Channel {
                            r#type: Box::new(TypeNode::Name {
                                package: None,
                                id: Span::new("u32", 29, 1),
                                args: vec![]
                            }),
                            direction: Some(ChannelDirection::Send),
                        }),
                        args: vec![ExprNode::Call(CallNode {
                            func: Box::new(ExprNode::Name(OperandNameNode {
                                package: None,
                                id: Span::new("h", 34, 1)
                            })),
                            type_arg: Some(TypeNode::Channel {
                                r#type: Box::new(TypeNode::Name {
                                    package: Some(Span::new("pkg", 43, 1)),
                                    id: Span::new("T", 47, 1),
                                    args: vec![
                                        TypeNode::Name {
                                            package: None,
                                            id: Span::new("E", 49, 1),
                                            args: vec![]
                                        },
                                        TypeNode::Name {
                                            package: Some(Span::new("x", 52, 1)),
                                            id: Span::new("F", 54, 1),
                                            args: vec![]
                                        }
                                    ]
                                }),
                                direction: Some(ChannelDirection::Send),
                            }),
                            args: vec![],
                            variadic: false,
                            location: 35..57,
                            annotation: None,
                        })],
                        variadic: false,
                        location: 21..58,
                        annotation: None,
                    })
                ],
                variadic: true,
                location: 3..62,
                annotation: None,
            }),
            // TODO: ... g(<-chan u32), when supported
            parse("p.f(chan int, '\\a', g(chan<- u32, h(chan<- pkg.T[E, x.F]))...)").unwrap()
        )
    }
}
