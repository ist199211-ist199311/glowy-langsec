use std::cmp::Ordering;

use parser::ast::{CallNode, ExprNode, FunctionDeclNode};

use super::{exprs::visit_expr, visit_statement};
use crate::{
    context::VisitFileContext,
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceType},
    symbols::Symbol,
};

pub fn visit_function_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &FunctionDeclNode<'a>,
) {
    context.symbol_table.push();

    for param in &node.signature.params {
        for id in &param.ids {
            if let Some(prev_symbol) = context.symbol_table.create_symbol(Symbol::new_with_package(
                context.current_package(),
                id.clone(),
                None, // TODO: make label depend on calls to function
                false,
            )) {
                context.report_error(
                    id.location(),
                    AnalysisError::Redeclaration {
                        file: context.file(),
                        prev_symbol: prev_symbol.name().clone(),
                        new_symbol: id.clone(),
                    },
                )
            }
        }
    }

    for statement in &node.body {
        visit_statement(context, statement);
    }

    context.symbol_table.pop();
}

pub fn visit_call<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &CallNode<'a>,
) -> Option<LabelBacktrace<'a>> {
    let branch_backtrace = context.branch_backtrace().cloned();

    let mut label = Label::from(&branch_backtrace);
    let args_backtraces: Vec<_> = node
        .args
        .iter()
        .flat_map(|arg| visit_expr(context, arg))
        .inspect(|backtrace| label = label.union(backtrace.label()))
        .chain(std::iter::once(branch_backtrace).flatten())
        .collect();

    let backtrace = LabelBacktrace::new(
        LabelBacktraceType::FunctionCall,
        context.file(),
        node.location.clone(),
        None,
        label.clone(),
        &args_backtraces,
    );

    if let Some(annotation) = &node.annotation {
        if annotation.scope == "sink" {
            let sink_label = Label::from_parts(&annotation.labels);

            if let None | Some(Ordering::Greater) = label.partial_cmp(&sink_label) {
                context.report_error(
                    node.location.clone(),
                    AnalysisError::InsecureFlow {
                        kind: InsecureFlowKind::Call,
                        sink_label,
                        backtrace: backtrace
                            .clone()
                            .expect("call label should not to be bottom"),
                    },
                );
            }
        } else {
            // TODO: error message
        }
    }

    backtrace
}

pub fn visit_return<'a>(context: &mut VisitFileContext<'a, '_>, exprs: &[ExprNode<'a>]) {
    todo!()
}
