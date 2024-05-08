use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::recognize,
    multi::{many1, many_m_n},
    sequence::terminated,
};

use crate::{ast::LiteralNode, NResult, Span};

fn decimal_int_lit(s: Span) -> NResult<LiteralNode> {
    let (s, caught) = recognize(many1(terminated(digit1, many_m_n(0, 1, char('_')))))(s)?;

    if let Ok(parsed) = caught.fragment().parse() {
        Ok((s, LiteralNode::Int(parsed)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Fail, // hack!
        )))
    }
}

// nom has float parser!

fn int_literal(s: Span) -> NResult<LiteralNode> {
    alt((decimal_int_lit,))(s)
}

pub fn literal(s: Span) -> NResult<LiteralNode> {
    alt((int_literal,))(s)
}
