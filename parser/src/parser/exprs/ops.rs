use crate::{
    ast::{BinaryOpKind, ExprNode, UnaryOpKind},
    parser::PResult,
    token::TokenKind,
    TokenStream,
};

// adapted from https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html

fn infix_binding_power(op: &BinaryOpKind) -> (u8, u8) {
    // (low, high) means left-to-right associativity
    match op {
        BinaryOpKind::LogicalOr => (1, 2),
        BinaryOpKind::LogicalAnd => (3, 4),
        BinaryOpKind::Eq
        | BinaryOpKind::NotEq
        | BinaryOpKind::Less
        | BinaryOpKind::LessEq
        | BinaryOpKind::Greater
        | BinaryOpKind::GreaterEq => (5, 6),
        BinaryOpKind::Sum
        | BinaryOpKind::Diff
        | BinaryOpKind::BitwiseOr
        | BinaryOpKind::BitwiseXor => (7, 8),
        BinaryOpKind::Product
        | BinaryOpKind::Quotient
        | BinaryOpKind::Remainder
        | BinaryOpKind::ShiftLeft
        | BinaryOpKind::ShiftRight
        | BinaryOpKind::BitwiseAnd
        | BinaryOpKind::BitClear => (9, 10),
    }
}

fn parse_unary<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    if let Some(token) = s.peek().cloned().transpose()? {
        if let Ok(op) = token.kind.try_into() {
            s.next(); // advance

            return Ok(ExprNode::UnaryOp {
                kind: op,
                operand: Box::new(parse_unary(s)?),
            });
        }
    }

    super::parse_primary_expression(s)
}

pub fn parse_expression_bp<'a>(s: &mut TokenStream<'a>, min_bp: u8) -> PResult<'a, ExprNode<'a>> {
    let mut lhs = parse_unary(s)?;

    while let Some(token) = s.peek().cloned().transpose()? {
        let op = match token.kind.try_into() {
            Ok(kind) => kind,
            Err(_) => break,
        };

        let (l_bp, r_bp) = infix_binding_power(&op);
        if l_bp < min_bp {
            // operator to the left of this one is stronger than us,
            // so we need to let the lhs go to be with them...
            break;
        }

        s.next(); // step past operator token
        let rhs = parse_expression_bp(s, r_bp)?;

        lhs = ExprNode::BinaryOp {
            kind: op,
            left: Box::new(lhs),
            right: Box::new(rhs),
        }
    }

    Ok(lhs)
}

pub struct UnknownOpKind;

impl TryFrom<TokenKind> for UnaryOpKind {
    type Error = UnknownOpKind;

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        let op = match kind {
            TokenKind::Plus => Self::Identity,
            TokenKind::Minus => Self::Negation,
            TokenKind::Caret => Self::Complement,
            TokenKind::Excl => Self::Not,
            TokenKind::Star => Self::Deref,
            TokenKind::Amp => Self::Address,
            TokenKind::LtMinus => Self::Receive,
            _ => return Err(UnknownOpKind),
        };

        Ok(op)
    }
}

impl TryFrom<TokenKind> for BinaryOpKind {
    type Error = UnknownOpKind;

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        let op = match kind {
            TokenKind::DoubleEq => Self::Eq,
            TokenKind::NotEq => Self::NotEq,
            TokenKind::Lt => Self::Less,
            TokenKind::LtEq => Self::LessEq,
            TokenKind::Gt => Self::Greater,
            TokenKind::GtEq => Self::GreaterEq,
            TokenKind::Plus => Self::Sum,
            TokenKind::Minus => Self::Diff,
            TokenKind::Star => Self::Product,
            TokenKind::Slash => Self::Quotient,
            TokenKind::Percent => Self::Remainder,
            TokenKind::DoubleLt => Self::ShiftLeft,
            TokenKind::DoubleGt => Self::ShiftRight,
            TokenKind::Pipe => Self::BitwiseOr,
            TokenKind::Amp => Self::BitwiseAnd,
            TokenKind::Caret => Self::BitwiseXor,
            TokenKind::AmpCaret => Self::BitClear,
            TokenKind::DoubleAmp => Self::LogicalAnd,
            TokenKind::DoublePipe => Self::LogicalOr,
            _ => return Err(UnknownOpKind),
        };

        Ok(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{LiteralNode, OperandNameNode},
        lexer::Lexer,
        Span,
    };

    fn parse(input: &str) -> PResult<'_, ExprNode<'_>> {
        let mut lexer = Lexer::new(input).peekable();

        parse_expression_bp(&mut lexer, 0)
    }

    #[test]
    fn precedence() {
        assert_eq!(
            ExprNode::BinaryOp {
                kind: BinaryOpKind::LogicalOr,
                left: Box::new(ExprNode::BinaryOp {
                    kind: BinaryOpKind::Sum,
                    left: Box::new(ExprNode::Literal(LiteralNode::Int(42))),
                    right: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Product,
                        left: Box::new(ExprNode::UnaryOp {
                            kind: UnaryOpKind::Negation,
                            operand: Box::new(ExprNode::Name(OperandNameNode {
                                package: None,
                                id: Span::new("a", 6, 1)
                            }))
                        }),
                        right: Box::new(ExprNode::Literal(LiteralNode::Int(3)))
                    })
                }),
                right: Box::new(ExprNode::BinaryOp {
                    kind: BinaryOpKind::LogicalAnd,
                    left: Box::new(ExprNode::Name(OperandNameNode {
                        package: None,
                        id: Span::new("b", 15, 1)
                    })),
                    right: Box::new(ExprNode::BinaryOp {
                        kind: BinaryOpKind::Eq,
                        left: Box::new(ExprNode::BinaryOp {
                            kind: BinaryOpKind::Eq,
                            left: Box::new(ExprNode::UnaryOp {
                                kind: UnaryOpKind::Identity,
                                operand: Box::new(ExprNode::Literal(LiteralNode::Int(2)))
                            }),
                            right: Box::new(ExprNode::Literal(LiteralNode::Int(4)))
                        }),
                        right: Box::new(ExprNode::BinaryOp {
                            kind: BinaryOpKind::BitwiseXor,
                            left: Box::new(ExprNode::UnaryOp {
                                kind: UnaryOpKind::Receive,
                                operand: Box::new(ExprNode::Literal(LiteralNode::Int(9)))
                            }),
                            right: Box::new(ExprNode::BinaryOp {
                                kind: BinaryOpKind::ShiftLeft,
                                left: Box::new(ExprNode::Literal(LiteralNode::Int(2))),
                                right: Box::new(ExprNode::Name(OperandNameNode {
                                    package: None,
                                    id: Span::new("abc", 42, 1)
                                }))
                            })
                        })
                    })
                })
            },
            parse("42 + -a * 3 || b && +2 == 4 == <-9 ^ 2 << abc").unwrap()
        );
    }

    #[test]
    fn parens() {
        assert_eq!(
            ExprNode::BinaryOp {
                kind: BinaryOpKind::Product,
                left: Box::new(ExprNode::Literal(LiteralNode::Int(2))),
                right: Box::new(ExprNode::BinaryOp {
                    kind: BinaryOpKind::Diff,
                    left: Box::new(ExprNode::Literal(LiteralNode::Int(3))),
                    right: Box::new(ExprNode::UnaryOp {
                        kind: UnaryOpKind::Address,
                        operand: Box::new(ExprNode::Name(OperandNameNode {
                            package: Some(Span::new("ab", 13, 2)),
                            id: Span::new("cd", 16, 2)
                        }))
                    }),
                })
            },
            parse("2 * \n (3 - &\tab.cd)").unwrap()
        );
    }
}
