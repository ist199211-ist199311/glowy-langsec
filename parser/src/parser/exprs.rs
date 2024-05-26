use self::postfix::parse_postfix_if_exists;
use super::{expect, PResult};
use crate::{
    ast::{ExprNode, LiteralNode, OperandNameNode},
    parser::of_kind,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

mod ops;
mod postfix;

fn parse_operand_name<'a>(s: &mut TokenStream<'a>) -> PResult<'a, OperandNameNode<'a>> {
    let token = expect(s, TokenKind::Ident, Some("operand name"))?;

    if let Some(Ok(of_kind!(TokenKind::Period))) = s.peek() {
        s.next(); // advance

        Ok(OperandNameNode {
            package: Some(token.span),
            id: expect(s, TokenKind::Ident, Some("operand name"))?.span,
        })
    } else {
        Ok(OperandNameNode {
            package: None,
            id: token.span,
        })
    }
}

pub fn parse_primary_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    let expr = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::Ident)) => parse_operand_name(s)?.into(),
        Some(of_kind!(TokenKind::Int(v))) => {
            s.next(); // advance

            LiteralNode::Int(v).into()
        }
        Some(of_kind!(TokenKind::ParenL)) => {
            s.next(); // advance
            let inner = parse_expression(s)?;
            expect(s, TokenKind::ParenR, Some("parenthesized expression"))?;

            inner
        }
        found => {
            return Err(ParsingError::UnexpectedConstruct {
                expected: "a primary expression",
                found,
            })
        }
    };

    parse_postfix_if_exists(s, expr)
}

pub fn parse_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    ops::parse_expression_bp(s, 0)
}

pub fn parse_expressions_list<'a, F, R, E>(
    s: &mut TokenStream<'a>,
    stop_cond: F,
) -> PResult<'a, Option<(Vec<ExprNode<'a>>, R)>>
where
    F: Fn(Token<'a>) -> Result<R, E>,
{
    let mut exprs = vec![];

    let mut over = false;

    while let Some(Ok(token)) = s.peek().cloned() {
        if let Ok(res) = stop_cond(token) {
            return Ok(Some((exprs, res)));
        }

        if over {
            // 2 non-comma-separated expressions in a row are not allowed
            expect(s, TokenKind::Comma, Some("expressions list"))?;
            // (^^ we know this will error, that's the point)
        }

        exprs.push(parse_expression(s)?);

        if let Some(Ok(of_kind!(TokenKind::Comma))) = s.peek() {
            s.next(); // advance
        } else {
            // the next token must be an assignment operator,
            // otherwise something's wrong -- this will be checked
            // at the beginning of the next loop iteration
            over = true;
        }
    }

    Ok(None)
}

pub fn parse_expressions_list_while<'a, F>(
    s: &mut TokenStream<'a>,
    cond: F,
) -> PResult<'a, Option<Vec<ExprNode<'a>>>>
where
    F: Fn(Token<'a>) -> bool,
{
    Ok(
        parse_expressions_list(s, |token| (!cond(token)).then_some(()).ok_or(()))?
            .map(|(exprs, _)| exprs),
    )
}
