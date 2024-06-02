use std::cmp::Ordering;

use parser::{
    ast::{CallNode, ExprNode, FunctionDeclNode},
    Location,
};

use super::{exprs::visit_expr, visit_statement};
use crate::{
    context::{FunctionOutcome, VisitFileContext},
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceKind},
    symbols::Symbol,
};

pub fn visit_function_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &FunctionDeclNode<'a>,
) {
    // declare function symbol
    let package = context.current_package();
    if let Some(prev_symbol) = context.symtab_mut().create_symbol(Symbol::new_with_package(
        package,
        node.name.clone(),
        None,
        false,
    )) {
        context.report_error(
            node.name.location(),
            AnalysisError::Redeclaration {
                file: context.file(),
                prev_symbol: prev_symbol.name().clone(),
                new_symbol: node.name.clone(),
            },
        )
    }

    context.symtab_mut().push();

    // ensure no stale return labels are present
    // TODO: support nested functions
    context.clear_return_backtraces();

    for param in &node.signature.params {
        for id in &param.ids {
            if let Some(prev_symbol) = context.symtab_mut().create_symbol(Symbol::new_with_package(
                package,
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

    let args_backtraces = node
        .signature
        .params
        .iter()
        .flat_map(|param| &param.ids)
        .map(|id| context.symtab().get_symbol(id.content()))
        .map(|symbol| symbol.and_then(|symbol| symbol.backtrace().clone()))
        .collect::<Vec<_>>();
    let outcome = FunctionOutcome {
        arguments: args_backtraces,
        return_value: context.get_return_backtraces().to_vec(),
    };
    context.clear_return_backtraces();

    // TODO: make labels depend on calls to functions
    let arg_labels = vec![Label::Bottom; node.signature.params.len()];

    context.set_function_outcome(package, node.name.content(), arg_labels, outcome);

    context.symtab_mut().pop();
}

pub fn visit_call<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &CallNode<'a>,
) -> Option<LabelBacktrace<'a>> {
    let branch_backtrace = context.branch_backtrace().cloned();

    let mut label = Label::from(&branch_backtrace);
    let mut args_labels = Vec::new();
    let args_backtraces: Vec<_> = node
        .args
        .iter()
        .flat_map(|arg| {
            let backtrace = visit_expr(context, arg);
            args_labels.push(Label::from(&backtrace));
            backtrace
        })
        .inspect(|backtrace| label = label.union(backtrace.label()))
        .chain(branch_backtrace)
        .collect();

    let backtrace = LabelBacktrace::new(
        LabelBacktraceKind::FunctionCall,
        context.file(),
        node.location.clone(),
        None,
        label.clone(),
        &args_backtraces,
    );

    if let Some(annotation) = &node.annotation {
        if annotation.scope == "sink" {
            let sink_label = Label::from_parts(&annotation.tags);

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

    if let ExprNode::Name(name) = node.func.as_ref() {
        if let Some(outcome) =
            context.get_function_outcome(context.current_package(), name.id.content(), &args_labels)
        {
            // TODO do something with outcome arguments...
            let label = outcome
                .return_value
                .iter()
                .map(|backtrace| backtrace.label())
                .fold(Label::Bottom, |acc, label| acc.union(label));
            return LabelBacktrace::new(
                LabelBacktraceKind::FunctionCall,
                context.file(),
                node.location.clone(),
                None,
                label,
                &outcome.return_value,
            );
        }
    }

    backtrace
}

pub fn visit_return<'a>(
    context: &mut VisitFileContext<'a, '_>,
    exprs: &[ExprNode<'a>],
    location: &Location,
) {
    let branch_backtrace = context.branch_backtrace().cloned();
    let exprs_backtraces: Vec<_> = exprs
        .iter()
        .flat_map(|node| visit_expr(context, node))
        .chain(branch_backtrace)
        .collect();
    let label = exprs_backtraces
        .iter()
        .map(|backtrace| backtrace.label())
        .fold(Label::Bottom, |acc, label| acc.union(label));

    let return_backtrace = LabelBacktrace::new(
        LabelBacktraceKind::Return,
        context.file(),
        location.clone(),
        None,
        label,
        &exprs_backtraces,
    );
    if let Some(backtrace) = return_backtrace {
        context.push_return_backtraces(backtrace);
    }
}
