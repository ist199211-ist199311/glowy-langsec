use nom::{
    branch::alt,
    character::complete::{char, multispace0},
    sequence::{pair, preceded, terminated},
};

use crate::{ast::OperandNameNode, identifier, NResult, Span};

fn simple_operand_name(s: Span) -> NResult<OperandNameNode> {
    let (s, id) = identifier(s)?;

    Ok((s, OperandNameNode { package: None, id }))
}

fn qualified_operand_name(s: Span) -> NResult<OperandNameNode> {
    let (s, (qualifier, id)) = pair(
        terminated(identifier, char('.')),
        preceded(multispace0, identifier),
    )(s)?;

    Ok((
        s,
        OperandNameNode {
            package: Some(qualifier),
            id,
        },
    ))
}

pub fn operand_name(s: Span) -> NResult<OperandNameNode> {
    alt((qualified_operand_name, simple_operand_name))(s)
}
