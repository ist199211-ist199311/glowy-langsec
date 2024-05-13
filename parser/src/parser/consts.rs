use super::{expect, exprs::parse_expression, of_kind, PResult};
use crate::{
    ast::{ConstDeclSpecNode, TopLevelDeclNode},
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

pub fn parse_const_spec<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ConstDeclSpecNode<'a>> {
    let mut ids = vec![];
    let mut exprs = vec![];

    loop {
        let token = expect(s, TokenKind::Ident, Some("list of identifiers"))?;
        ids.push(token.span);

        match s.next().transpose()? {
            Some(of_kind!(TokenKind::Assign)) => break,
            Some(of_kind!(TokenKind::Comma)) => {}
            found => {
                return Err(ParsingError::UnexpectedTokenKind {
                    expected: TokenKind::Comma,
                    found,
                    context: Some("list of identifiers"),
                })
            }
        };
    }

    exprs.push(parse_expression(s)?);
    while exprs.len() < ids.len() {
        expect(s, TokenKind::Comma, Some("list of expressions"))?;
        exprs.push(parse_expression(s)?);
    }

    Ok(ConstDeclSpecNode::try_new(ids, exprs).unwrap())
}

pub fn parse_const_specs_list<'a>(
    s: &mut TokenStream<'a>,
) -> PResult<'a, Vec<ConstDeclSpecNode<'a>>> {
    expect(s, TokenKind::ParenL, Some("constant declaration"))?;

    // could be simplified, but spec allows for an empty list... `const ();`

    let mut specs = vec![];
    loop {
        match s.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::ParenR)) => break,
            Some(of_kind!(TokenKind::Ident)) => {
                specs.push(parse_const_spec(s)?);
                expect(s, TokenKind::SemiColon, Some("constant specification"))?;
            }
            found => {
                return Err(ParsingError::UnexpectedConstruct {
                    expected: "a constant specification",
                    found,
                })
            }
        };
    }

    s.next(); // consume )

    Ok(specs)
}

pub fn parse_const_decl<'a>(s: &mut TokenStream<'a>) -> PResult<'a, TopLevelDeclNode<'a>> {
    expect(s, TokenKind::Const, Some("constant declaration"))?;

    let specs = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::Ident)) => vec![parse_const_spec(s)?],
        Some(of_kind!(TokenKind::ParenL)) => parse_const_specs_list(s)?,
        found => {
            return Err(ParsingError::UnexpectedConstruct {
                expected: "a constant specification",
                found,
            })
        }
    };

    Ok(TopLevelDeclNode::Const(specs))
}
