use parser::{
    ast::{ExprNode, IndexingNode},
    Span,
};

use super::funcs::visit_call;
use crate::{
    context::VisitFileContext,
    errors::AnalysisError,
    labels::{LabelBacktrace, LabelBacktraceType},
};

pub fn visit_expr<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &ExprNode<'a>,
) -> Vec<LabelBacktrace<'a>> {
    match node {
        ExprNode::Name(name) => match context
            .symtab()
            .get_symbol_label_backtrace(name.id.content())
        {
            Some(label_backtrace) => label_backtrace
                .as_ref()
                .map(|backtrace| {
                    LabelBacktrace::new(
                        LabelBacktraceType::Expression,
                        context.file(),
                        name.id.clone(),
                        backtrace.label().clone(),
                        std::iter::once(backtrace),
                    )
                })
                .into_iter()
                .flatten()
                .collect(),
            None => {
                context.report_error(
                    name.id.location(),
                    AnalysisError::UnknownSymbol {
                        file: context.file(),
                        symbol: name.id.clone(),
                    },
                );
                vec![]
            }
        },
        ExprNode::Literal(_) => vec![],
        ExprNode::UnaryOp { operand, .. } => visit_expr(context, operand.as_ref()),
        ExprNode::BinaryOp { left, right, .. } => {
            let mut llabel = visit_expr(context, left.as_ref());
            let rlabel = visit_expr(context, right.as_ref());
            llabel.extend(rlabel);
            llabel
        }
        ExprNode::Call(call) => visit_call(context, call),
        ExprNode::Indexing(indexing) => visit_indexing(context, indexing),
    }
}

fn visit_indexing<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &IndexingNode<'a>,
) -> Vec<LabelBacktrace<'a>> {
    todo!()
}

pub fn find_first_ident<'a>(node: &ExprNode<'a>) -> Option<Span<'a>> {
    match node {
        ExprNode::Name(name) => Some(name.id.clone()),
        ExprNode::Literal(_) => None,
        ExprNode::Call(call_node) => find_first_ident(&call_node.func),
        ExprNode::Indexing(indexing_node) => find_first_ident(&indexing_node.expr),
        ExprNode::UnaryOp { operand, .. } => find_first_ident(operand),
        ExprNode::BinaryOp { left, right, .. } => {
            find_first_ident(left).or_else(|| find_first_ident(right))
        }
    }
}
