use decls::try_parse_top_level_decl;
use imports::try_parse_import;

use crate::{
    ast::{PackageClauseNode, SourceFileNode},
    errors::ParsingError,
    stream::TokenStream,
    token::{Token, TokenKind},
};

mod decls;
mod exprs;
mod imports;
mod stmts;
mod types;

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

pub fn parse_source_file<'a>(s: &mut TokenStream<'a>) -> PResult<'a, SourceFileNode<'a>> {
    let package_clause = parse_package_clause(s)?;

    expect(s, TokenKind::SemiColon, None)?;

    let mut imports = vec![];
    while let Some(import) = try_parse_import(s)? {
        imports.push(import);
        expect(s, TokenKind::SemiColon, None)?;
    }

    let mut top_level_decls = vec![];
    while let Some(decl) = try_parse_top_level_decl(s)? {
        top_level_decls.push(decl);
        expect(s, TokenKind::SemiColon, None)?;
    }

    Ok(SourceFileNode {
        package_clause,
        imports,
        top_level_decls,
    })
}

// utility to allow non-committal peeking deep within a token stream
pub struct BacktrackingContext<'l, 'a> {
    // rust enforces that invoker can't use original while we hold it here
    // until the last time the context is used, since otherwise there would
    // be 2 mutable references at once
    original: &'l mut TokenStream<'a>,
    clone: TokenStream<'a>,
}

impl<'l, 'a> BacktrackingContext<'l, 'a> {
    pub fn new(original: &'l mut TokenStream<'a>) -> Self {
        let clone = original.clone();

        Self { original, clone }
    }

    pub fn stream(&mut self) -> &mut TokenStream<'a> {
        &mut self.clone
    }

    pub fn commit(&mut self) -> PResult<'a, ()> {
        fn offset<'a>(s: &mut TokenStream<'a>) -> PResult<'a, Option<usize>> {
            Ok(s.peek()
                .cloned()
                .transpose()?
                .map(|token| token.span.offset))
        }

        if let Some(target) = offset(&mut self.clone)? {
            while let Some(current) = offset(self.original)? {
                if current == target {
                    break;
                }
                self.original.next();
            }
        } else {
            // clone was consumed until end-of-file
            self.original.last();
        }

        Ok(())
    }
}
