use self::{
    concur::parse_go_statement,
    flow::{parse_if_statement, parse_return_statement},
};
use super::{
    decls::bindings::{parse_const_decl, parse_var_decl},
    expect,
    exprs::{parse_expression, parse_expressions_list, parse_expressions_list_while},
    PResult,
};
use crate::{
    ast::{
        AssignmentKind, AssignmentNode, BlockNode, ExprNode, SendNode, ShortVarDeclNode,
        StatementNode,
    },
    parser::{of_kind, BacktrackingContext},
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

mod concur;
mod flow;

// continue from the right-hand side
fn resume_parsing_assignment_rhs<'a>(
    s: &mut TokenStream<'a>,
    lhs: Vec<ExprNode<'a>>,
    kind: AssignmentKind,
) -> PResult<'a, StatementNode<'a>> {
    if let Some(rhs) = parse_expressions_list_while(s, |t| !terminal_token(&t.kind))? {
        Ok(StatementNode::Assignment(AssignmentNode { kind, lhs, rhs }))
    } else {
        // reached end-of-file...
        expect(s, TokenKind::SemiColon, Some("assignment"))?;
        // ^^ this will error
        unreachable!()
    }
}

// continue from the left-hand side
fn resume_parsing_assignment_lhs<'a>(
    s: &mut TokenStream<'a>,
    mut lhs: Vec<ExprNode<'a>>,
) -> PResult<'a, StatementNode<'a>> {
    // collect the rest of the expressions, if any
    if let Some((rest, kind)) = parse_expressions_list(s, |t| AssignmentKind::try_from(t.kind))? {
        s.next(); // step over operator

        lhs.extend(rest);
        resume_parsing_assignment_rhs(s, lhs, kind)
    } else {
        // reached end-of-file and found no assignment operator...
        Err(ParsingError::UnexpectedConstruct {
            expected: "an assignment statement",
            found: None, // if we got here, this must mean end-of-file
        })
    }
}

// statements that start with an expression and then diverge wrt operator
fn parse_expression_first_stmt<'a>(s: &mut TokenStream<'a>) -> PResult<'a, StatementNode<'a>> {
    let lhs = parse_expression(s)?;

    // this needs to be separate so we don't consume the semicolon,
    // and to avoid using peek on the match (would require .next in every branch)
    if let Some(Ok(of_kind!(kind))) = s.peek() {
        if terminal_token(kind) {
            return Ok(StatementNode::Expr(lhs));
        }
    }

    let node = match s.next().transpose()? {
        Some(of_kind!(TokenKind::LtMinus)) => StatementNode::Send(SendNode {
            channel: lhs,
            expr: parse_expression(s)?,
        }),
        Some(of_kind!(TokenKind::PlusPlus)) => StatementNode::Inc(lhs),
        Some(of_kind!(TokenKind::MinusMinus)) => StatementNode::Dec(lhs),
        Some(of_kind!(TokenKind::Comma)) => resume_parsing_assignment_lhs(s, vec![lhs])?,
        found => {
            if let Some(token) = found.clone() {
                if let Ok(kind) = AssignmentKind::try_from(token.kind) {
                    return resume_parsing_assignment_rhs(s, vec![lhs], kind);
                }
            }

            return Err(ParsingError::UnexpectedTokenKind {
                expected: TokenKind::SemiColon,
                found,
                context: Some("statement"),
            });
        }
    };

    Ok(node)
}

fn parse_identifier_first_stmt<'a>(s: &mut TokenStream<'a>) -> PResult<'a, StatementNode<'a>> {
    let mut context = BacktrackingContext::new(s);
    let b = context.stream();

    let first = expect(b, TokenKind::Ident, Some("statement"))?;

    // assume it's a short var decl and that we're collecting ids (vs expressions)
    let mut ids = vec![first.span];

    let mut was_comma = false; // whether the last token was a comma

    loop {
        match b.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::Ident)) if was_comma => {
                if was_comma {
                    ids.push(b.next().unwrap()?.span);
                    was_comma = false;
                } else {
                    // 2 identifiers in a row
                    expect(b, TokenKind::Comma, Some("statement"))?;
                    // ^^ this will error
                }
            }
            found @ Some(of_kind!(TokenKind::Comma)) => {
                if was_comma {
                    // 2 commas in a row
                    return Err(ParsingError::UnexpectedConstruct {
                        expected: "an identifier or an expression",
                        found,
                    });
                } else {
                    b.next(); // advance
                    was_comma = true;
                }
            }
            Some(of_kind!(TokenKind::ColonAssign)) if !was_comma => break, // short var decl!

            // we got it wrong... they're expressions
            _ => return parse_expression_first_stmt(s), // backtrack
        }
    }

    b.next(); // step over operator that caused break
    context.commit()?; // we're sure it's a short var decl so we can go back to the main stream now

    if let Some(exprs) = parse_expressions_list_while(s, |t| !terminal_token(&t.kind))? {
        Ok(StatementNode::ShortVarDecl(ShortVarDeclNode { ids, exprs }))
    } else {
        // reached end-of-file...
        expect(s, TokenKind::SemiColon, Some("short variable declaration"))?;
        // ^^ this will error
        unreachable!()
    }
}

fn parse_statement<'a>(
    s: &mut TokenStream<'a>,
    allow_non_simple: bool,
) -> PResult<'a, StatementNode<'a>> {
    let node = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::SemiColon)) => StatementNode::Empty,
        Some(of_kind!(TokenKind::CurlyL)) if allow_non_simple => {
            StatementNode::Block(parse_block(s)?)
        }
        Some(of_kind!(TokenKind::If)) if allow_non_simple => parse_if_statement(s)?.into(),
        Some(of_kind!(TokenKind::Return)) if allow_non_simple => parse_return_statement(s)?,
        Some(of_kind!(TokenKind::Go)) if allow_non_simple => parse_go_statement(s)?,

        // declarations (sadly cannot be abstracted, indistinguishable if not for keywords)
        Some(of_kind!(TokenKind::Const)) => parse_const_decl(s)?.into(),
        Some(of_kind!(TokenKind::Var)) => parse_var_decl(s)?.into(),

        Some(of_kind!(TokenKind::Ident)) => parse_identifier_first_stmt(s)?,
        _ => parse_expression_first_stmt(s)?,
    };

    Ok(node)
}

pub fn parse_block<'a>(s: &mut TokenStream<'a>) -> PResult<'a, BlockNode<'a>> {
    expect(s, TokenKind::CurlyL, Some("block"))?;

    let mut stmts = vec![];

    while !matches!(s.peek(), Some(Ok(of_kind!(TokenKind::CurlyR)))) {
        stmts.push(parse_statement(s, true)?);

        // spec allows omitting semicolon before }
        if let Some(Ok(of_kind!(TokenKind::CurlyR))) = s.peek() {
            break;
        }

        expect(s, TokenKind::SemiColon, Some("block"))?;
    }

    expect(s, TokenKind::CurlyR, Some("block"))?;

    Ok(stmts)
}

// may terminate a statement
fn terminal_token(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::SemiColon | TokenKind::CurlyR)
}

pub struct UnknownAssignmentKind;

impl TryFrom<TokenKind> for AssignmentKind {
    type Error = UnknownAssignmentKind;

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        let res = match kind {
            TokenKind::Assign => Self::Simple,
            TokenKind::PlusAssign => Self::Sum,
            TokenKind::MinusAssign => Self::Diff,
            TokenKind::StarAssign => Self::Product,
            TokenKind::SlashAssign => Self::Quotient,
            TokenKind::PercentAssign => Self::Remainder,
            TokenKind::DoubleLtAssign => Self::ShiftLeft,
            TokenKind::DoubleGtAssign => Self::ShiftRight,
            TokenKind::PipeAssign => Self::BitwiseOr,
            TokenKind::AmpAssign => Self::BitwiseAnd,
            TokenKind::CaretAssign => Self::BitwiseXor,
            TokenKind::AmpCaretAssign => Self::BitClear,
            _ => return Err(UnknownAssignmentKind),
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{BinaryOpKind, LiteralNode, OperandNameNode, UnaryOpKind},
        lexer::Lexer,
        Span,
    };

    fn parse(input: &str) -> PResult<'_, BlockNode<'_>> {
        let mut lexer = Lexer::new(input).peekable();

        parse_block(&mut lexer)
    }

    #[test]
    fn block() {
        assert_eq!(
            vec![
                StatementNode::Expr(ExprNode::BinaryOp {
                    kind: BinaryOpKind::Sum,
                    left: Box::new(ExprNode::Literal(LiteralNode::Int(2))),
                    right: Box::new(ExprNode::Literal(LiteralNode::Int(7)))
                }),
                StatementNode::Empty,
                StatementNode::Assignment(AssignmentNode {
                    kind: AssignmentKind::Simple,
                    lhs: vec![
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("a", 88, 5)
                        }),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("b", 91, 5)
                        })
                    ],
                    rhs: vec![
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("c", 95, 5)
                        }),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("d", 98, 5)
                        })
                    ]
                }),
                StatementNode::Assignment(AssignmentNode {
                    kind: AssignmentKind::Simple,
                    lhs: vec![
                        ExprNode::UnaryOp {
                            kind: UnaryOpKind::Negation,
                            operand: Box::new(ExprNode::Literal(LiteralNode::Int(4)))
                        },
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("x", 125, 6)
                        }),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("k", 129, 6)
                        })
                    ],
                    rhs: vec![
                        ExprNode::BinaryOp {
                            kind: BinaryOpKind::Product,
                            left: Box::new(ExprNode::Literal(LiteralNode::Int(4))),
                            right: Box::new(ExprNode::Literal(LiteralNode::Int(2)))
                        },
                        ExprNode::BinaryOp {
                            kind: BinaryOpKind::Sum,
                            left: Box::new(ExprNode::Literal(LiteralNode::Int(5))),
                            right: Box::new(ExprNode::Literal(LiteralNode::Int(2)))
                        },
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("x", 148, 6)
                        })
                    ]
                }),
                StatementNode::ShortVarDecl(ShortVarDeclNode {
                    ids: vec![
                        Span::new("k", 171, 7),
                        Span::new("r", 174, 7),
                        Span::new("v", 177, 7)
                    ],
                    exprs: vec![
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("m", 182, 7)
                        }),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("n", 185, 7)
                        }),
                        ExprNode::Name(OperandNameNode {
                            package: None,
                            id: Span::new("o", 188, 7)
                        })
                    ]
                }),
                StatementNode::Assignment(AssignmentNode {
                    kind: AssignmentKind::Simple,
                    lhs: vec![ExprNode::Name(OperandNameNode {
                        package: None,
                        id: Span::new("a", 211, 8)
                    })],
                    rhs: vec![ExprNode::Name(OperandNameNode {
                        package: None,
                        id: Span::new("b", 215, 8)
                    })]
                }),
                StatementNode::ShortVarDecl(ShortVarDeclNode {
                    ids: vec![Span::new("c", 238, 9)],
                    exprs: vec![ExprNode::Name(OperandNameNode {
                        package: None,
                        id: Span::new("d", 243, 9)
                    })]
                })
            ],
            parse(
                "
                {
                    2 + 7;
                    ;
                    a, b = c, d;
                    -4, x, (k) = 4 * 2, 5 + 2, x;
                    k, r, v := m, n, o;
                    a = b;
                    c := d;
                }
            "
            )
            .unwrap()
        )
    }
}
