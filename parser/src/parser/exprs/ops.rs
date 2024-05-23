use crate::{
    ast::{BinaryOpKind, ExprNode},
    parser::PResult,
    token::TokenKind,
    TokenStream,
};

// adapted from https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html

fn infix_binding_power(op: &BinaryOpKind) -> (u8, u8) {
    // (low, high) means left-to-right associativity
    match op {
        BinaryOpKind::LogicalOr => (1, 2),
        BinaryOpKind::LogicalAnd => (3, 4),
        BinaryOpKind::Eq
        | BinaryOpKind::NotEq
        | BinaryOpKind::Less
        | BinaryOpKind::LessEq
        | BinaryOpKind::Greater
        | BinaryOpKind::GreaterEq => (5, 6),
        BinaryOpKind::Sum
        | BinaryOpKind::Diff
        | BinaryOpKind::BitwiseOr
        | BinaryOpKind::BitwiseXor => (7, 8),
        BinaryOpKind::Product
        | BinaryOpKind::Quotient
        | BinaryOpKind::Remainder
        | BinaryOpKind::ShiftLeft
        | BinaryOpKind::ShiftRight
        | BinaryOpKind::BitwiseAnd
        | BinaryOpKind::BitClear => (9, 10),
    }
}

pub fn parse_expression_bp<'a>(s: &mut TokenStream<'a>, min_bp: u8) -> PResult<'a, ExprNode<'a>> {
    let mut lhs = super::parse_primary_expression(s)?;

    while let Some(token) = s.peek().cloned().transpose()? {
        let op = match token.kind.try_into() {
            Ok(kind) => kind,
            Err(_) => break,
        };

        let (l_bp, r_bp) = infix_binding_power(&op);
        if l_bp < min_bp {
            // operator to the left of this one is stronger than us,
            // so we need to let the lhs go to be with them...
            break;
        }

        s.next(); // step past operator token
        let rhs = parse_expression_bp(s, r_bp)?;

        lhs = ExprNode::BinaryOp {
            kind: op,
            left: Box::new(lhs),
            right: Box::new(rhs),
        }
    }

    Ok(lhs)
}

impl TryFrom<TokenKind> for BinaryOpKind {
    type Error = ();

    fn try_from(kind: TokenKind) -> Result<Self, Self::Error> {
        let op = match kind {
            TokenKind::DoubleEq => Self::Eq,
            TokenKind::NotEq => Self::NotEq,
            TokenKind::Lt => Self::Less,
            TokenKind::LtEq => Self::LessEq,
            TokenKind::Gt => Self::Greater,
            TokenKind::GtEq => Self::GreaterEq,
            TokenKind::Plus => Self::Sum,
            TokenKind::Minus => Self::Diff,
            TokenKind::Star => Self::Product,
            TokenKind::Slash => Self::Quotient,
            TokenKind::Percent => Self::Remainder,
            TokenKind::DoubleLt => Self::ShiftLeft,
            TokenKind::DoubleGt => Self::ShiftRight,
            TokenKind::Pipe => Self::BitwiseOr,
            TokenKind::Amp => Self::BitwiseAnd,
            TokenKind::Caret => Self::BitwiseXor,
            TokenKind::AmpCaret => Self::BitClear,
            TokenKind::DoubleAmp => Self::LogicalAnd,
            TokenKind::DoublePipe => Self::LogicalOr,
            _ => return Err(()),
        };

        Ok(op)
    }
}
