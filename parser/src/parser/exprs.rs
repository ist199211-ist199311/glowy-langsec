use super::{expect, PResult};
use crate::{
    ast::{ExprNode, LiteralNode, OperandNameNode},
    parser::of_kind,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

mod ops;

fn parse_operand_name<'a>(s: &mut TokenStream<'a>) -> PResult<'a, OperandNameNode<'a>> {
    let token = expect(s, TokenKind::Ident, Some("operand name"))?;

    if let Some(Ok(of_kind!(TokenKind::Period))) = s.peek() {
        s.next(); // advance

        Ok(OperandNameNode {
            package: Some(token.span),
            id: expect(s, TokenKind::Ident, Some("operand name"))?.span,
        })
    } else {
        Ok(OperandNameNode {
            package: None,
            id: token.span,
        })
    }
}

pub fn parse_primary_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::Ident)) => Ok(parse_operand_name(s)?.into()),
        Some(of_kind!(TokenKind::Int(v))) => {
            s.next(); // advance
            Ok(LiteralNode::Int(v).into())
        }
        Some(of_kind!(TokenKind::ParenL)) => {
            s.next(); // advance
            let inner = parse_expression(s)?;
            expect(s, TokenKind::ParenR, Some("parenthesized expression"))?;
            Ok(inner)
        }
        found => Err(ParsingError::UnexpectedConstruct {
            expected: "a primary expression",
            found,
        }),
    }
}

pub fn parse_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    ops::parse_expression_bp(s, 0)
}
