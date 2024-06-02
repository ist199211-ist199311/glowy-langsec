use super::{expect, of_kind, PResult};
use crate::{
    ast::{ImportNode, ImportSpecNode},
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

fn parse_spec<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ImportSpecNode<'a>> {
    let identifier = match s.peek() {
        Some(Ok(of_kind!(TokenKind::Ident))) | Some(Ok(of_kind!(TokenKind::Period))) => {
            let token = s.next().unwrap()?; // advance
            Some(token.span)
        }
        _ => None,
    };

    match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::String(path))) => {
            s.next(); // advance
            Ok(ImportSpecNode { identifier, path })
        }
        found => Err(ParsingError::UnexpectedConstruct {
            expected: "an import specification",
            found,
        }),
    }
}

fn parse_specs_list<'a>(s: &mut TokenStream<'a>) -> PResult<'a, Vec<ImportSpecNode<'a>>> {
    expect(s, TokenKind::ParenL, Some("import declaration"))?;

    // could be simplified, but spec allows for an empty list... `import ();`

    let mut specs = vec![];
    loop {
        match s.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::ParenR)) => break,
            Some(of_kind!(TokenKind::Ident))
            | Some(of_kind!(TokenKind::Period))
            | Some(of_kind!(TokenKind::String(_))) => {
                specs.push(parse_spec(s)?);

                // spec allows omitting semicolon before closing (
                if let Some(Ok(of_kind!(TokenKind::ParenR))) = s.peek() {
                    break;
                }

                expect(s, TokenKind::SemiColon, Some("an import specification"))?;
            }
            found => {
                return Err(ParsingError::UnexpectedConstruct {
                    expected: "an import specification",
                    found,
                })
            }
        };
    }

    s.next(); // consume )

    Ok(specs)
}

pub fn try_parse_import<'a>(s: &mut TokenStream<'a>) -> PResult<'a, Option<ImportNode<'a>>> {
    if let Some(Ok(of_kind!(TokenKind::Import))) = s.peek() {
        s.next(); // advance

        let specs = match s.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::Ident))
            | Some(of_kind!(TokenKind::Period))
            | Some(of_kind!(TokenKind::String(_))) => vec![parse_spec(s)?],
            Some(of_kind!(TokenKind::ParenL)) => parse_specs_list(s)?,
            found => {
                return Err(ParsingError::UnexpectedConstruct {
                    expected: "an import specification",
                    found,
                })
            }
        };

        Ok(Some(ImportNode { specs }))
    } else {
        Ok(None)
    }
}
