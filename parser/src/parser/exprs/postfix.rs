use super::{parse_expression, parse_expressions_list_while};
use crate::{
    ast::{CallNode, ExprNode, IndexingNode},
    parser::{expect, of_kind, PResult},
    token::{Token, TokenKind},
    TokenStream,
};

pub fn parse_call<'a>(s: &mut TokenStream<'a>, func: ExprNode<'a>) -> PResult<'a, CallNode<'a>> {
    expect(s, TokenKind::ParenL, Some("function call"))?;

    // TODO: support trailing comma

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
        args,
        variadic,
    })
}

pub fn parse_indexing<'a>(
    s: &mut TokenStream<'a>,
    expr: ExprNode<'a>,
) -> PResult<'a, IndexingNode<'a>> {
    expect(s, TokenKind::SquareL, Some("indexing expression"))?;

    let index = parse_expression(s)?;

    // optional trailing comma
    if let Some(Ok(of_kind!(TokenKind::Comma))) = s.peek() {
        s.next(); // advance
    }

    expect(s, TokenKind::SquareR, Some("indexing expression"))?;

    Ok(IndexingNode {
        expr: Box::new(expr),
        index: Box::new(index),
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
        ast::{BinaryOpKind, LiteralNode, OperandNameNode, UnaryOpKind},
        lexer::Lexer,
        parser::exprs::parse_expression,
        Span,
    };

    fn parse(input: &str) -> PResult<'_, ExprNode<'_>> {
        let mut lexer = Lexer::new(input).peekable();

        parse_expression(&mut lexer)
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
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(14)))
                    }),
                    args: vec![
                        ExprNode::BinaryOp {
                            kind: BinaryOpKind::Sum,
                            left: Box::new(ExprNode::Literal(LiteralNode::Int(21))),
                            right: Box::new(ExprNode::BinaryOp {
                                kind: BinaryOpKind::Product,
                                left: Box::new(ExprNode::Literal(LiteralNode::Int(7))),
                                right: Box::new(ExprNode::UnaryOp {
                                    kind: UnaryOpKind::Negation,
                                    operand: Box::new(ExprNode::Literal(LiteralNode::Int(9)))
                                })
                            }),
                        },
                        ExprNode::Literal(LiteralNode::Int(0))
                    ],
                    variadic: true
                })),
                args: vec![],
                variadic: false
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
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(14)))
                    }),
                    index: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Sum,
                        left: Box::new(ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("k", 15, 1)
                        })),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(2))),
                    }),
                })),
                args: vec![],
                variadic: false
            }),
            parse("(abc.def + 14)[k + 2,]()").unwrap()
        )
    }
}
