use bindings::parse_binding_decl;

use self::bindings::BindingKind;
use super::{of_kind, PResult};
use crate::{
    ast::TopLevelDeclNode,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

mod bindings;

pub fn try_parse_top_level_decl<'a>(
    s: &mut TokenStream<'a>,
) -> PResult<'a, Option<TopLevelDeclNode<'a>>> {
    match s.peek().cloned().transpose()? {
        None => Ok(None), // eof
        Some(of_kind!(TokenKind::Const)) => Ok(Some(parse_binding_decl(s, BindingKind::Const)?)),
        Some(of_kind!(TokenKind::Var)) => Ok(Some(parse_binding_decl(s, BindingKind::Var)?)),
        Some(token) => Err(ParsingError::UnexpectedConstruct {
            expected: "a top-level declaration",
            found: Some(token),
        }),
    }
}
