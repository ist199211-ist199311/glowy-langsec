use super::{expect, PResult};
use crate::{ast::ExprNode, token::TokenKind, TokenStream};

pub fn parse_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    // TODO: actually parse expressions

    let token = expect(s, TokenKind::Ident, Some("expression"))?;

    Ok(ExprNode::Placeholder(token.span))
}
