use consts::parse_const_decl;

use crate::{
    ast::{PackageClauseNode, SourceFileNode, TopLevelDeclNode},
    errors::ParsingError,
    token::{Token, TokenKind},
    TokenStream,
};

mod consts;
mod exprs;

type PResult<'a, T> = Result<T, ParsingError<'a>>;

macro_rules! of_kind {
    ($k:pat) => {
        Token { kind: $k, .. }
    };
}
// required to allow the `allow()` below
#[allow(clippy::useless_attribute)]
// required for usage in this module's children
#[allow(clippy::needless_pub_self)]
pub(self) use of_kind;

fn expect<'a>(
    s: &mut TokenStream<'a>,
    kind: TokenKind,
    context: Option<&'static str>,
) -> PResult<'a, Token<'a>> {
    let found = if let Some(token) = s.next() {
        let token = token?;
        if token.kind == kind {
            return Ok(token);
        } else {
            Some(token)
        }
    } else {
        // eof
        None
    };

    Err(ParsingError::UnexpectedTokenKind {
        expected: kind,
        found,
        context,
    })
}

fn parse_package_clause<'a>(s: &mut TokenStream<'a>) -> PResult<'a, PackageClauseNode<'a>> {
    expect(s, TokenKind::Package, Some("beginning of source file"))?;

    let ident = expect(s, TokenKind::Ident, Some("package clause"))?;

    Ok(PackageClauseNode { id: ident.span })
}

fn try_parse_top_level_decl<'a>(
    s: &mut TokenStream<'a>,
) -> PResult<'a, Option<TopLevelDeclNode<'a>>> {
    match s.peek().cloned().transpose()? {
        None => Ok(None), // eof
        Some(of_kind!(TokenKind::Const)) => Ok(Some(parse_const_decl(s)?)),
        Some(token) => Err(ParsingError::UnexpectedConstruct {
            expected: "a top-level declaration",
            found: Some(token),
        }),
    }
}

pub fn parse_source_file<'a>(s: &mut TokenStream<'a>) -> PResult<'a, SourceFileNode<'a>> {
    let package_clause = parse_package_clause(s)?;

    expect(s, TokenKind::SemiColon, None)?;

    let mut top_level_decls = vec![];

    while let Some(decl) = try_parse_top_level_decl(s)? {
        top_level_decls.push(decl);
        expect(s, TokenKind::SemiColon, None)?;
    }

    Ok(SourceFileNode {
        package_clause,
        top_level_decls,
    })
}
