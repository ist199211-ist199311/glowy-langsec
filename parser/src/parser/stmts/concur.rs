use crate::{
    ast::StatementNode,
    parser::{expect, exprs::parse_expression, PResult},
    token::TokenKind,
    TokenStream,
};

pub fn parse_go_statement<'a>(s: &mut TokenStream<'a>) -> PResult<'a, StatementNode<'a>> {
    expect(s, TokenKind::Go, Some("go statement"))?;

    // technically parenthesized expressions are illegal here, but...
    let expr = parse_expression(s)?;

    Ok(StatementNode::Go(expr))
}
