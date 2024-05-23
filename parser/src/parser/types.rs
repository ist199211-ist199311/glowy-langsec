use super::{expect, of_kind, PResult};
use crate::{
    ast::TypeNode,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

pub fn parse_type_args<'a>(s: &mut TokenStream<'a>) -> PResult<'a, Vec<TypeNode<'a>>> {
    let mut args = vec![];

    if !matches!(s.peek(), Some(Ok(of_kind!(TokenKind::SquareL)))) {
        return Ok(args);
    }

    s.next(); // advance

    loop {
        if !args.is_empty() {
            expect(s, TokenKind::Comma, Some("list of type arguments"))?;

            // if what we just read was actually an optional trailing comma
            // and now the list is over, abort reading a new type
            if let Some(Ok(of_kind!(TokenKind::SquareR))) = s.peek() {
                s.next(); // advance
                break;
            }
        }

        args.push(parse_type(s)?);

        if !matches!(s.peek(), Some(Ok(of_kind!(TokenKind::Comma)))) {
            break;
        }
    }

    expect(s, TokenKind::SquareR, Some("type arguments"))?;

    Ok(args)
}

pub fn parse_type_name<'a>(s: &mut TokenStream<'a>) -> PResult<'a, TypeNode<'a>> {
    let token = expect(s, TokenKind::Ident, Some("type name"))?;

    if let Some(Ok(of_kind!(TokenKind::Period))) = s.peek() {
        s.next(); // advance

        Ok(TypeNode::Name {
            package: Some(token.span),
            id: expect(s, TokenKind::Ident, Some("type name"))?.span,
            args: parse_type_args(s)?,
        })
    } else {
        Ok(TypeNode::Name {
            package: None,
            id: token.span,
            args: parse_type_args(s)?,
        })
    }
}

pub fn parse_type<'a>(s: &mut TokenStream<'a>) -> PResult<'a, TypeNode<'a>> {
    match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::ParenL)) => {
            s.next(); // advance
            let inner = parse_type(s)?;
            expect(s, TokenKind::ParenR, Some("parenthesized type"))?;
            Ok(inner)
        }
        Some(of_kind!(TokenKind::Ident)) => parse_type_name(s),
        found => Err(ParsingError::UnexpectedConstruct {
            expected: "a type",
            found,
        }),
    }
}
