use super::{expect, PResult};
use crate::{
    ast::{ExprNode, OperandNameNode},
    parser::of_kind,
    token::{Token, TokenKind},
    ParsingError, TokenStream,
};

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

pub fn parse_expression<'a>(s: &mut TokenStream<'a>) -> PResult<'a, ExprNode<'a>> {
    match s.peek().cloned().transpose()? {
        Some(of_kind!(TokenKind::Ident)) => Ok(parse_operand_name(s)?.into()),
        found => Err(ParsingError::UnexpectedConstruct {
            expected: "an expression",
            found,
        }),
    }
}
