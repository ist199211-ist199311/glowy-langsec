use crate::{
    ast::{FunctionDeclNode, FunctionParamDeclNode, FunctionResultNode, FunctionSignatureNode},
    parser::{expect, of_kind, types::parse_type, PResult},
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

fn parse_param_decl<'a>(s: &mut TokenStream<'a>) -> PResult<'a, FunctionParamDeclNode<'a>> {
    let mut ids = vec![];

    // TODO: support declarations without identifiers, e.g. "int"
    // this is non-trivial because a declaration "a, b int" is
    // indistinguishable from 2 declarations "a" and "b int"
    // (this example is illegal but the ambiguity remains within a single
    // declaration) note: partial support already exists but only when variadic
    // or type is not simple

    while let Some(token) = s.peek().cloned().transpose()? {
        if ids.is_empty() && token.kind != TokenKind::Ident {
            // this is definitely not a name, so it must be a type
            break;
        }

        ids.push(expect(s, TokenKind::Ident, Some("parameter declaration"))?.span);

        // check the next token
        if let Some(Ok(of_kind!(TokenKind::Comma))) = s.peek() {
            s.next(); // advance

            // read another identifier
            continue;
        }

        // next element is a type
        break;
    }

    let variadic = if let Some(Ok(of_kind!(TokenKind::Ellipsis))) = s.peek() {
        s.next(); // advance

        true
    } else {
        false
    };

    let r#type = parse_type(s)?;

    Ok(FunctionParamDeclNode {
        ids,
        variadic,
        r#type,
    })
}

fn parse_params<'a>(s: &mut TokenStream<'a>) -> PResult<'a, Vec<FunctionParamDeclNode<'a>>> {
    expect(s, TokenKind::ParenL, Some("function parameters"))?;

    let mut params = vec![];

    loop {
        // this should be a while but it's not easy to express the condition
        if let Some(Ok(of_kind!(TokenKind::ParenR))) = s.peek() {
            break;
        }

        params.push(parse_param_decl(s)?);

        // need to check again in case there isn't an (optional) trailing comma
        if let Some(Ok(of_kind!(TokenKind::ParenR))) = s.peek() {
            break;
        }

        expect(s, TokenKind::Comma, Some("parameter list"))?;
    }

    expect(s, TokenKind::ParenR, Some("function parameters"))?;

    Ok(params)
}

fn parse_signature<'a>(s: &mut TokenStream<'a>) -> PResult<'a, FunctionSignatureNode<'a>> {
    let params = parse_params(s)?;

    let result = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::CurlyL)) => None,
        Some(of_kind!(TokenKind::ParenL)) => Some(FunctionResultNode::Params(parse_params(s)?)),
        _ => Some(FunctionResultNode::Single(parse_type(s)?)),
    };

    Ok(FunctionSignatureNode { params, result })
}

pub fn parse_function_decl<'a>(s: &mut TokenStream<'a>) -> PResult<'a, FunctionDeclNode<'a>> {
    expect(s, TokenKind::Func, Some("function declaration"))?;

    let name = expect(s, TokenKind::Ident, Some("function name"))?.span;

    if let Some(Ok(of_kind!(TokenKind::SquareL))) = s.peek() {
        // TODO: support type parameters
        return Err(ParsingError::UnexpectedConstruct {
            expected: "function signature",
            found: s.next().transpose()?,
        });
    }

    let signature = parse_signature(s)?;

    // TODO: support body
    let body = vec![];

    Ok(FunctionDeclNode {
        name,
        signature,
        body,
    })
}
