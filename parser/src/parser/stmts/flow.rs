use crate::{
    ast::{ElseNode, IfNode},
    parser::{expect, exprs::parse_expression, of_kind, stmts::parse_block, PResult},
    token::{Token, TokenKind},
    TokenStream,
};

pub fn parse_if_statement<'a>(s: &mut TokenStream<'a>) -> PResult<'a, IfNode<'a>> {
    expect(s, TokenKind::If, Some("if statement"))?;

    // TODO: support simple statements to execute before condition

    let cond = parse_expression(s)?;
    let then = parse_block(s)?;

    let otherwise = if let Some(Ok(of_kind!(TokenKind::Else))) = s.peek() {
        s.next(); // advance

        let node = if let Some(Ok(of_kind!(TokenKind::If))) = s.peek() {
            ElseNode::If(Box::new(parse_if_statement(s)?))
        } else {
            ElseNode::Block(parse_block(s)?)
        };

        Some(node)
    } else {
        None
    };

    Ok(IfNode {
        cond,
        then,
        otherwise,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{
            AssignmentKind, AssignmentNode, BinaryOpKind, BlockNode, ExprNode, LiteralNode,
            OperandNameNode, ShortVarDeclNode, StatementNode, UnaryOpKind,
        },
        lexer::Lexer,
        parser::stmts::parse_block,
        Span,
    };

    fn parse(input: &str) -> PResult<'_, BlockNode<'_>> {
        let mut lexer = Lexer::new(input).peekable();

        parse_block(&mut lexer)
    }

    #[test]
    fn if_chain() {
        assert_eq!(
            vec![StatementNode::If(IfNode {
                cond: ExprNode::BinaryOp {
                    kind: BinaryOpKind::Greater,
                    left: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Sum,
                        left: Box::new(ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("a", 50, 3)
                        })),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(3)))
                    }),
                    right: Box::new(ExprNode::Literal(LiteralNode::Int(4)))
                },
                then: vec![
                    StatementNode::Empty,
                    StatementNode::Assignment(AssignmentNode {
                        kind: AssignmentKind::Simple,
                        lhs: vec![ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("a", 120, 5)
                        })],
                        rhs: vec![ExprNode::Literal(LiteralNode::Int(4))]
                    })
                ],
                otherwise: Some(ElseNode::If(Box::new(IfNode {
                    cond: ExprNode::UnaryOp {
                        kind: UnaryOpKind::Negation,
                        operand: Box::new(ExprNode::UnaryOp {
                            kind: UnaryOpKind::Negation,
                            operand: Box::new(ExprNode::Literal(LiteralNode::Int(9)))
                        })
                    },
                    then: vec![StatementNode::ShortVarDecl(ShortVarDeclNode {
                        ids: vec![Span::new("k", 197, 7)],
                        exprs: vec![ExprNode::Literal(LiteralNode::Int(3))]
                    })],
                    otherwise: Some(ElseNode::Block(vec![
                        StatementNode::Block(vec![]),
                        StatementNode::Dec(ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("m", 298, 10)
                        })),
                        StatementNode::Assignment(AssignmentNode {
                            kind: AssignmentKind::BitClear,
                            lhs: vec![
                                ExprNode::Name(OperandNameNode {
                                    package: None,
                                    id: Span::new("k", 331, 11)
                                }),
                                ExprNode::Name(OperandNameNode {
                                    package: Some(Span::new("m", 334, 11)),
                                    id: Span::new("r", 336, 11)
                                })
                            ],
                            rhs: vec![
                                ExprNode::Literal(LiteralNode::Int(3)),
                                ExprNode::Literal(LiteralNode::Int(2)),
                            ]
                        })
                    ]))
                })))
            })],
            parse(
                "
                    {
                        if a + 3 > 4 {
                            ;
                            a = 4;
                        } else if -(-9) {
                            k := 3;
                        } else {
                            {};
                            m--;
                            k, m.r &^= 3, 2;
                        };
                    }
                ",
            )
            .unwrap(),
        )
    }
}
