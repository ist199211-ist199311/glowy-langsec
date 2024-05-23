use crate::{
    ast::{BindingDeclSpecNode, TopLevelDeclNode},
    parser::{expect, exprs::parse_expression, of_kind, types::parse_type, PResult},
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

// bindings is our term for constants and variables,
// since their declarations look the same, allowing code reuse

pub enum BindingKind {
    Const,
    Var,
}

impl BindingKind {
    fn keyword(&self) -> TokenKind {
        match self {
            Self::Const => TokenKind::Const,
            Self::Var => TokenKind::Var,
        }
    }

    fn decl_context(&self) -> &'static str {
        match self {
            Self::Const => "constant declaration",
            Self::Var => "variable declaration",
        }
    }

    fn spec_construct(&self) -> &'static str {
        match self {
            Self::Const => "a constant specification",
            Self::Var => "a variable specification",
        }
    }

    fn spec_context(&self) -> &'static str {
        match self {
            Self::Const => "constant specification",
            Self::Var => "variable specification",
        }
    }

    fn build_node<'a>(&self, specs: Vec<BindingDeclSpecNode<'a>>) -> TopLevelDeclNode<'a> {
        match self {
            Self::Const => TopLevelDeclNode::Const(specs),
            Self::Var => TopLevelDeclNode::Var(specs),
        }
    }
}

fn parse_spec<'a>(
    s: &mut TokenStream<'a>,
    kind: &BindingKind,
) -> PResult<'a, BindingDeclSpecNode<'a>> {
    let mut ids = vec![];
    let mut exprs = vec![];
    let mut r#type = None;

    loop {
        let token = expect(s, TokenKind::Ident, Some("list of identifiers"))?;
        ids.push(token.span);

        match s.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::Comma)) => {
                s.next(); // advance
                continue;
            }
            Some(of_kind!(TokenKind::Assign)) => {
                s.next(); // advance
            }
            Some(_) => {
                r#type = Some(parse_type(s)?);
                expect(s, TokenKind::Assign, Some(kind.spec_context()))?;
            }
            None => {
                return Err(ParsingError::UnexpectedTokenKind {
                    expected: TokenKind::Comma,
                    found: None,
                    context: Some("list of identifiers"),
                })
            }
        };

        break;
    }

    // TODO: allow empty expression list for consts (if prev spec was non-empty)

    exprs.push(parse_expression(s)?);
    while exprs.len() < ids.len() {
        expect(s, TokenKind::Comma, Some("list of expressions"))?;
        exprs.push(parse_expression(s)?);
    }

    Ok(BindingDeclSpecNode::try_new(ids, exprs, r#type).unwrap())
}

fn parse_specs_list<'a>(
    s: &mut TokenStream<'a>,
    kind: &BindingKind,
) -> PResult<'a, Vec<BindingDeclSpecNode<'a>>> {
    expect(s, TokenKind::ParenL, Some(kind.decl_context()))?;

    // could be simplified, but spec allows for an empty list... `const ();`

    let mut specs = vec![];
    loop {
        match s.peek().cloned().transpose()? {
            Some(of_kind!(TokenKind::ParenR)) => break,
            Some(of_kind!(TokenKind::Ident)) => {
                specs.push(parse_spec(s, kind)?);
                expect(s, TokenKind::SemiColon, Some(kind.spec_context()))?;
            }
            found => {
                return Err(ParsingError::UnexpectedConstruct {
                    expected: kind.spec_construct(),
                    found,
                })
            }
        };
    }

    s.next(); // consume )

    Ok(specs)
}

pub fn parse_binding_decl<'a>(
    s: &mut TokenStream<'a>,
    kind: BindingKind,
) -> PResult<'a, TopLevelDeclNode<'a>> {
    expect(s, kind.keyword(), Some(kind.decl_context()))?;

    let specs = match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::Ident)) => vec![parse_spec(s, &kind)?],
        Some(of_kind!(TokenKind::ParenL)) => parse_specs_list(s, &kind)?,
        found => {
            return Err(ParsingError::UnexpectedConstruct {
                expected: kind.spec_construct(),
                found,
            })
        }
    };

    Ok(kind.build_node(specs))
}
