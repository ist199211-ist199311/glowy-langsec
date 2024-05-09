use crate::{
    ast::{PackageClauseNode, SourceFileNode},
    errors::ParsingError,
    token::{Token, TokenKind},
    TokenStream,
};

type PResult<'a, T> = Result<T, ParsingError<'a>>;

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

    Err(ParsingError::UnexpectedToken {
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

    if let Some(extra) = s.next() {
        Err(ParsingError::ExtraneousToken(extra?))
    } else {
        Ok(SourceFileNode { package_clause })
    }
}
