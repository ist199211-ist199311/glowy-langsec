use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, space0},
    sequence::{delimited, pair, tuple},
};

use crate::{
    ast::{BinaryOpKind, ExprNode, UnaryOpKind},
    exprs::expression,
    NResult, Span,
};

use super::primary_expression;

macro_rules! unary_op {
    ($s:expr, $operand:expr, $( ($tag:literal, $kind:expr) ),+ ) => {
        {
            let (s, (op, expr)) = pair(
                delimited(
                    space0,
                    alt((
                        $( tag($tag), )+
                    )),
                    multispace0
                ),
                $operand
            )($s)?;

            let kind = match *op.fragment() {
                $( $tag => $kind, )+
                tag => unreachable!("Unknown unary operator {tag}"),
            };

            (s, kind, expr)
        }
    };
}

macro_rules! binary_op {
    ($s:expr, $operand:expr, $( ($tag:literal, $kind:expr) ),+ ) => {
        {
            let (s, (left, op, right)) = tuple((
                $operand,
                delimited(
                    space0,
                    alt((
                        $( tag($tag), )+
                    )),
                    multispace0
                ),
                $operand
            ))($s)?;

            let kind = match *op.fragment() {
                $( $tag => $kind, )+
                tag => unreachable!("Unknown binary operator {tag}"),
            };

            (s, kind, left, right)
        }
    };
}

fn unary_operation(s: Span) -> NResult<ExprNode> {
    let (s, kind, expr) = unary_op!(
        s,
        expression,
        ("+", UnaryOpKind::Identity),
        ("-", UnaryOpKind::Negation),
        ("^", UnaryOpKind::Complement),
        ("!", UnaryOpKind::Not),
        ("*", UnaryOpKind::Deref),
        ("&", UnaryOpKind::Address),
        ("<-", UnaryOpKind::Receive)
    );

    Ok((
        s,
        ExprNode::UnaryOp {
            kind,
            operand: Box::new(expr),
        },
    ))
}

pub fn unary_or_primary_expr(s: Span) -> NResult<ExprNode> {
    alt((unary_operation, primary_expression))(s)
}

fn precedence_5_binary_op(s: Span) -> NResult<ExprNode> {
    let (s, kind, left, right) = binary_op!(
        s,
        expression,
        ("*", BinaryOpKind::Product),
        ("/", BinaryOpKind::Quotient),
        ("%", BinaryOpKind::Remainder),
        ("<<", BinaryOpKind::ShiftLeft),
        (">>", BinaryOpKind::ShiftRight),
        ("&", BinaryOpKind::BitwiseAnd),
        ("&^", BinaryOpKind::BitClear)
    );

    Ok((
        s,
        ExprNode::BinaryOp {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

fn precedence_4_binary_op(s: Span) -> NResult<ExprNode> {
    let (s, kind, left, right) = binary_op!(
        s,
        alt((precedence_5_binary_op, expression)),
        ("+", BinaryOpKind::Sum),
        ("-", BinaryOpKind::Diff),
        ("|", BinaryOpKind::BitwiseOr),
        ("^", BinaryOpKind::BitwiseXor)
    );

    Ok((
        s,
        ExprNode::BinaryOp {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

fn precedence_3_binary_op(s: Span) -> NResult<ExprNode> {
    let (s, kind, left, right) = binary_op!(
        s,
        alt((precedence_4_binary_op, expression)),
        ("==", BinaryOpKind::Eq),
        ("!=", BinaryOpKind::NotEq),
        ("<", BinaryOpKind::Less),
        ("<=", BinaryOpKind::LessEq),
        (">", BinaryOpKind::Greater),
        (">=", BinaryOpKind::GreaterEq)
    );

    Ok((
        s,
        ExprNode::BinaryOp {
            kind,
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

fn precedence_2_binary_op(s: Span) -> NResult<ExprNode> {
    let (s, (left, _, right)) = tuple((
        alt((precedence_3_binary_op, expression)),
        delimited(space0, tag("&&"), multispace0),
        alt((precedence_3_binary_op, expression)),
    ))(s)?;

    Ok((
        s,
        ExprNode::BinaryOp {
            kind: BinaryOpKind::LogicalAnd,
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

// aka. precedence_1_binary_op
pub fn binary_operation(s: Span) -> NResult<ExprNode> {
    let (s, (left, _, right)) = tuple((
        alt((precedence_2_binary_op, expression)),
        delimited(space0, tag("||"), multispace0),
        alt((precedence_2_binary_op, expression)),
    ))(s)?;

    Ok((
        s,
        ExprNode::BinaryOp {
            kind: BinaryOpKind::LogicalOr,
            left: Box::new(left),
            right: Box::new(right),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{ExprNode, LiteralNode, OperandNameNode},
        tests::{assert_eq_result, span},
    };

    #[test]
    fn unary() {
        assert_eq_result(
            ExprNode::UnaryOp {
                kind: UnaryOpKind::Identity,
                operand: Box::new(ExprNode::Name(OperandNameNode {
                    package: None,
                    id: span("x", 1, 1),
                })),
            },
            unary_operation(Span::new("+x")),
        )
    }

    #[test]
    fn binary() {
        assert_eq_result(
            ExprNode::BinaryOp {
                kind: BinaryOpKind::Diff,
                left: Box::new(ExprNode::BinaryOp {
                    kind: BinaryOpKind::Sum,
                    left: Box::new(ExprNode::Literal(LiteralNode::Int(42))),
                    right: Box::new(ExprNode::Name(OperandNameNode {
                        package: None,
                        id: span("a", 5, 1),
                    })),
                }),
                right: Box::new(ExprNode::Name(OperandNameNode {
                    package: None,
                    id: span("b", 9, 1),
                })),
            },
            // FIXME: this won't work because bin_op REQUIRES ||,
            // each level needs to optional check for theirs and otherwise delegate
            binary_operation(Span::new("42 + a - b")),
        )
    }
}
