use nom::{branch::alt, character::complete::char, combinator::map, sequence::delimited};

use crate::{ast::ExprNode, msp00, NResult, Span};

mod literals;
mod names;
mod ops;

pub fn operand(s: Span) -> NResult<ExprNode> {
    alt((
        delimited(char('('), msp00(expression), char(')')),
        map(names::operand_name, ExprNode::from),
        map(literals::literal, ExprNode::from),
    ))(s)
}

pub fn primary_expression(s: Span) -> NResult<ExprNode> {
    alt((
        operand,
        // TODO: more...
    ))(s)
}

pub fn expression(s: Span) -> NResult<ExprNode> {
    alt((ops::unary_or_primary_expr, ops::binary_operation))(s)
}
