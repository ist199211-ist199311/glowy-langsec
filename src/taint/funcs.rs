use std::cmp::Ordering;

use parser::ast::{CallNode, ExprNode, FunctionDeclNode};

use super::{exprs::find_first_ident, visit_expr, visit_statement};
use crate::{
    context::VisitFileContext,
    errors::AnalysisError,
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
) -> Vec<LabelBacktrace<'a>> {
    let mut label = Label::Bottom;
    let label_backtraces: Vec<_> = node
        .args
        .iter()
        .flat_map(|arg| visit_expr(context, arg))
        .inspect(|backtrace| label = label.union(backtrace.label()))
        .collect();

    // TODO handle this properly by providing a span for expressions in the parser
    let symbol = find_first_ident(&node.func)
        .expect("glowy currently only supports calling functions by their identifiers");
    let label_backtrace = LabelBacktrace::new(
        LabelBacktraceType::FunctionCall,
        context.file(),
        symbol.clone(),
        label.clone(),
        &label_backtraces,
    );

    if let Some(annotation) = &node.annotation {
        match annotation.scope {
            "sink" => {
                let sink_label = Label::from_parts(&annotation.labels);
                match label.partial_cmp(&sink_label) {
                    None | Some(Ordering::Greater) => context.report_error(
                        symbol.location(),
                        AnalysisError::DataFlowFuncCall {
                            sink_label,
                            label_backtrace: label_backtrace
                                .clone()
                                .expect("label of function call to not be bottom"),
                        },
                    ),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    label_backtrace.into_iter().collect()
}

pub fn visit_return<'a>(context: &mut VisitFileContext<'a, '_>, exprs: &[ExprNode<'a>]) {
    todo!()
}
