use crate::{
    ast::{BlockNode, SendNode, StatementNode},
    parser::of_kind,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

use super::{expect, exprs::parse_expression, PResult};

// statements that start with an expression and then diverge according to operator
fn parse_expression_first_stmt<'a>(s: &mut TokenStream<'a>) -> PResult<'a, StatementNode<'a>> {
    let lhs = parse_expression(s)?;

    let node = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::SemiColon)) => StatementNode::Expr(lhs),
        Some(of_kind!(TokenKind::LtMinus)) => StatementNode::Send(SendNode {
            channel: lhs,
            expr: parse_expression(s)?,
        }),
        Some(of_kind!(TokenKind::PlusPlus)) => {
            s.next(); // advance
            StatementNode::Inc(lhs)
        }
        Some(of_kind!(TokenKind::MinusMinus)) => {
            s.next(); // advance
            StatementNode::Dec(lhs)
        }
        found => {
            return Err(ParsingError::UnexpectedTokenKind {
                expected: TokenKind::SemiColon,
                found,
                context: Some("statement"),
            })
        }
    };

    Ok(node)
}

fn parse_statement<'a>(s: &mut TokenStream<'a>) -> PResult<'a, StatementNode<'a>> {
    let node = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::SemiColon)) => StatementNode::Empty,
        Some(of_kind!(TokenKind::CurlyL)) => StatementNode::Block(parse_block(s)?),
        // TODO: Ident => parse assignment or short var decl; defer to expression_first
        _ => parse_expression_first_stmt(s)?,
    };

    Ok(node)
}

pub fn parse_block<'a>(s: &mut TokenStream<'a>) -> PResult<'a, BlockNode<'a>> {
    expect(s, TokenKind::CurlyL, Some("block"))?;

    let mut stmts = vec![];

    while !matches!(s.peek(), Some(Ok(of_kind!(TokenKind::CurlyR)))) {
        stmts.push(parse_statement(s)?);
        expect(s, TokenKind::SemiColon, Some("block"))?;
    }

    expect(s, TokenKind::CurlyR, Some("block"))?;

    Ok(stmts)
}
