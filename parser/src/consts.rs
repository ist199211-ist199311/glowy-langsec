use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0},
    combinator::map,
    multi::{many_m_n, separated_list0, separated_list1},
    sequence::{delimited, preceded, terminated},
};

use crate::{ast::ConstDeclSpecNode, eol, identifier, msp00, msp01, NResult, Span};

fn const_spec(s: Span) -> NResult<ConstDeclSpecNode> {
    // TODO: support type
    // TODO: support initialization expressions
    let (s, ids) = separated_list1(tag(","), msp00(identifier))(s)?;
    let (s, _) = preceded(
        terminated(tag("="), multispace0),
        separated_list1(msp00(tag(",")), alphanumeric1), // FIXME: expr, not alpha!
    )(s)?;

    Ok((s, ConstDeclSpecNode { ids }))
}

pub fn const_decl(s: Span) -> NResult<Vec<ConstDeclSpecNode>> {
    delimited(
        msp01(tag("const")),
        alt((
            map(const_spec, |spec| vec![spec]),
            delimited(
                tag("("),
                // vv why is this not list1... I thought Google had vowed not to be evil...
                separated_list0(eol, const_spec),
                terminated(many_m_n(0, 1, msp00(tag(";"))), tag(")")),
            ),
        )),
        eol,
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{assert_eq_result, span};

    #[test]
    fn simple() {
        assert_eq_result(
            vec![ConstDeclSpecNode {
                ids: vec![span("a", 6, 1)],
            }],
            const_decl(Span::new("const a = 3")),
        )
    }

    #[test]
    fn multi() {
        assert_eq_result(
            vec![
                ConstDeclSpecNode {
                    ids: vec![span("X3", 9, 2)],
                },
                ConstDeclSpecNode {
                    ids: vec![span("m", 17, 3), span("n", 20, 3)],
                },
            ],
            const_decl(Span::new("const\t\n (X3 = 4\n m, n\t= 9, 6;)")),
        );
    }
}
