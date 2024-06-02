use parser::{
    ast::{CallNode, ExprNode, IndexingNode, OperandNameNode},
    Location,
};

use super::funcs::visit_call;
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
            let symbol = context.symtab().get_symbol(name.id.content());

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
                context.report_error(
                    name.id.location(),
                    AnalysisError::UnknownSymbol {
                        file: context.file(),
                        symbol: name.id.clone(),
                    },
                );

                None
            }
        }
        ExprNode::Literal(_) => None,
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

pub fn visit_indexing<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &IndexingNode<'a>,
) -> Option<LabelBacktrace<'a>> {
    todo!()
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
