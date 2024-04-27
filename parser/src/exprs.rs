use nom::{branch::alt, combinator::map};

use crate::{ast::ExprNode, NResult, Span};

mod literals;
mod names;

pub fn expression(s: Span) -> NResult<ExprNode> {
    alt((
        map(names::operand_name, ExprNode::from),
        map(literals::literal, ExprNode::from),
    ))(s)
}
