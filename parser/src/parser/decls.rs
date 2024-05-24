use bindings::{parse_const_decl, parse_var_decl};
use funcs::parse_function_decl;

use super::{of_kind, PResult};
use crate::{
    ast::TopLevelDeclNode,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

mod bindings;
mod funcs;

pub fn try_parse_top_level_decl<'a>(
    s: &mut TokenStream<'a>,
) -> PResult<'a, Option<TopLevelDeclNode<'a>>> {
    match s.peek().cloned().transpose()? {
        None => Ok(None), // eof
        Some(of_kind!(TokenKind::Const)) => Ok(Some(parse_const_decl(s)?)),
        Some(of_kind!(TokenKind::Var)) => Ok(Some(parse_var_decl(s)?)),
        Some(of_kind!(TokenKind::Func)) => Ok(Some(parse_function_decl(s).map(Into::into)?)),
        Some(token) => Err(ParsingError::UnexpectedConstruct {
            expected: "a top-level declaration",
            found: Some(token),
        }),
    }
}
