use parser::{
    ast::{CallNode, ExprNode, IndexingNode, OperandNameNode, UnaryOpKind},
    Location,
};

use super::{channels::visit_receive, funcs::visit_call, package_or_current};
use crate::{
    context::VisitFileContext,
    errors::AnalysisError,
    labels::{LabelBacktrace, LabelBacktraceKind},
};

pub fn visit_expr<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &ExprNode<'a>,
) -> Option<LabelBacktrace<'a>> {
    match node {
        ExprNode::Name(name) => {
            let package = package_or_current!(context, name.package);
            if let Some(current_symbol) = context.current_symbol() {
                // ensure reverse dependencies are accounted for
                if !context.symtab().is_local(package, name.id.content()) {
                    context.add_symbol_reverse_dependency(
                        (context.current_package(), current_symbol),
                        (package, name.id.content()),
                    );
                }
            }

            let symbol = context.symtab().get_symbol(package, name.id.content());

            if let Some(symbol) = symbol {
                symbol.backtrace().as_ref().map(|symbol_backtrace| {
                    LabelBacktrace::new(
                        LabelBacktraceKind::Expression,
                        context.file(),
                        name.id.location(),
                        Some(name.id.clone()),
                        symbol_backtrace.label().clone(),
                        std::iter::once(symbol_backtrace),
                    )
                    .unwrap() // safe since we know backtrace exists
                })
            } else {
                context.report_error(AnalysisError::UnknownSymbol {
                    file: context.file(),
                    symbol: name.id.clone(),
                });

                None
            }
        }
        ExprNode::Literal(_) => None,
        ExprNode::UnaryOp {
            kind: UnaryOpKind::Receive,
            operand,
            location,
        } => visit_receive(context, operand, location),
        ExprNode::UnaryOp { operand, .. } => visit_expr(context, operand.as_ref()),
        ExprNode::BinaryOp {
            left,
            right,
            location,
            ..
        } => {
            let left = visit_expr(context, left.as_ref());
            let right = visit_expr(context, right.as_ref());

            match (&left, &right) {
                (None, None) => None,
                (Some(_), None) => left,
                (None, Some(_)) => right,
                (Some(l), Some(r)) => {
                    Some(l.union(r, LabelBacktraceKind::Expression, location.clone(), None))
                }
            }
        }
        ExprNode::Call(call) => visit_call(context, call),
        ExprNode::Indexing(indexing) => visit_indexing(context, indexing),
    }
}

fn visit_indexing<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &IndexingNode<'a>,
) -> Option<LabelBacktrace<'a>> {
    let backtrace_expr = visit_expr(context, &node.expr);
    let backtrace_index = visit_expr(context, &node.index);

    match (&backtrace_expr, &backtrace_index) {
        (None, None) => None,
        (Some(_), None) => backtrace_expr,
        (None, Some(_)) => backtrace_index,
        (Some(left), Some(right)) => Some(left.union(
            right,
            LabelBacktraceKind::Expression,
            node.location.clone(),
            None,
        )),
    }
}

pub fn find_expr_location(node: &ExprNode<'_>) -> Option<Location> {
    let loc = match node {
        ExprNode::Name(OperandNameNode { id, .. }) => id.location(),
        ExprNode::Literal(_) => return None,
        ExprNode::Call(CallNode { location, .. }) => location.clone(),
        ExprNode::Indexing(IndexingNode { location, .. }) => location.clone(),
        ExprNode::UnaryOp { location, .. } => location.clone(),
        ExprNode::BinaryOp { location, .. } => location.clone(),
    };

    Some(loc)
}
