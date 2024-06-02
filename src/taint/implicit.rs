use parser::{
    ast::{AssignmentKind, AssignmentNode, CallNode, ElseNode, ExprNode, IfNode, LiteralNode},
    Location,
};

use super::{explicit::visit_assignment, exprs::visit_expr, visit_statement};
use crate::{
    context::VisitFileContext,
    labels::{LabelBacktrace, LabelBacktraceKind},
};

pub fn visit_if<'a>(context: &mut VisitFileContext<'a, '_>, node: &IfNode<'a>) {
    let pushed = if let Some(backtrace) = visit_expr(context, &node.cond) {
        context.push_branch_label(
            LabelBacktrace::new(
                LabelBacktraceKind::Branch,
                backtrace.file(),
                backtrace.location().clone(),
                None,
                backtrace.label().clone(),
                &[backtrace],
            )
            .unwrap(), // safe since the original backtrace exists
        );

        true
    } else {
        false
    };

    // Go spec: each if, for and switch is considered to be in its own implicit
    // block
    context.symtab_mut().push();

    context.symtab_mut().push();
    for statement in &node.then {
        visit_statement(context, statement);
    }
    context.symtab_mut().pop();

    match &node.otherwise {
        Some(ElseNode::If(else_if)) => visit_if(context, else_if),
        Some(ElseNode::Block(stmts)) => {
            context.symtab_mut().push();
            for stmt in stmts {
                visit_statement(context, stmt);
            }
            context.symtab_mut().pop();
        }
        None => {}
    }

    context.symtab_mut().pop(); // implicit block

    if pushed {
        context.pop_branch_label();
    }
}

// TODO: visit_for

pub fn visit_incdec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    operand: &ExprNode<'a>,
    location: &Location,
) {
    // handled here as syntactic sugar for assignment

    visit_assignment(
        context,
        &AssignmentNode {
            kind: AssignmentKind::Sum, // can be anything but Simple
            lhs: vec![operand.clone()],
            rhs: vec![ExprNode::Literal(LiteralNode::Int(1))],
            location: location.clone(),
        },
    )
}

pub fn visit_go<'a>(context: &mut VisitFileContext<'a, '_>, expr: &CallNode<'a>) {
    todo!()
}
