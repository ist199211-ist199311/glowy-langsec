use std::cmp::Ordering;

use parser::{
    ast::{CallNode, ExprNode, FunctionDeclNode},
    Location,
};

use super::{
    exprs::{find_symbols_in_expr, visit_expr},
    package_or_current, visit_statement,
};
use crate::{
    context::{FunctionOutcome, VisitFileContext},
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceKind, LabelTag},
    symbols::Symbol,
};

pub fn visit_function_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &FunctionDeclNode<'a>,
) {
    let package = context.current_package();

    // declaration of global symbols is responsibility of the declarations visitor
    if context.is_in_global_symbol() {
        if let Some(prev_symbol) = context.symtab_mut().create_symbol(Symbol::new_with_package(
            package,
            node.name.clone(),
            None,
            false,
        )) {
            context.report_error(AnalysisError::Redeclaration {
                file: context.file(),
                prev_symbol: prev_symbol.name().clone(),
                new_symbol: node.name.clone(),
            })
        }
    }

    let func_name = node.name.content();
    if !context.should_visit_global_symbol(package, func_name) {
        return;
    }

    context.symtab_mut().push();

    // ensure no stale return labels are present
    // TODO: support nested functions
    context.clear_return_backtraces();
    context.enter_global_symbol(context.current_package(), func_name);

    let mut param_id = 0;

    for param in &node.signature.params {
        for id in &param.ids {
            let param_backtrace = LabelBacktrace::new(
                LabelBacktraceKind::FunctionArgument,
                context.file(),
                id.location(),
                Some(id.clone()),
                Label::from_synthetic_id(param_id),
                &[],
            );
            param_id += 1;
            if let Some(prev_symbol) = context.symtab_mut().create_symbol(Symbol::new_with_package(
                package,
                id.clone(),
                param_backtrace,
                false,
            )) {
                context.report_error(AnalysisError::Redeclaration {
                    file: context.file(),
                    prev_symbol: prev_symbol.name().clone(),
                    new_symbol: id.clone(),
                })
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
        .map(|id| context.symtab().get_symbol(package, id.content()))
        .enumerate()
        .map(|(i, symbol)| {
            symbol.and_then(|symbol| {
                symbol
                    .backtrace()
                    .clone()
                    .and_then(|backtrace| backtrace.remove_tag(&LabelTag::Synthetic(i)))
            })
        })
        .collect::<Vec<_>>();
    let outcome = FunctionOutcome {
        arguments: args_backtraces,
        return_value: context.get_return_backtraces().to_vec(),
    };
    context.clear_return_backtraces();

    if context.set_function_outcome(package, func_name, outcome) {
        // outcome has changed, propagate to dependencies
        context.enqueue_symbol_reverse_dependencies(package, func_name);
    }

    context.leave_global_symbol();
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
                context.report_error(AnalysisError::InsecureFlow {
                    kind: InsecureFlowKind::Call,
                    sink_label,
                    backtrace: backtrace
                        .clone()
                        .expect("call args label should not be bottom"),
                });
            }
        } else {
            // TODO: error message
        }
    }

    if let ExprNode::Name(name) = node.func.as_ref() {
        let func_package = package_or_current!(context, name.package);
        let is_local = context.symtab().is_local(func_package, name.id.content());
        if !is_local {
            if let Some(current_symbol) = context.current_symbol() {
                context.add_symbol_reverse_dependency(
                    (context.current_package(), current_symbol),
                    (func_package, name.id.content()),
                );
            }
        }
        if let Some(outcome) = context.get_function_outcome(func_package, name.id.content()) {
            let label = outcome
                .return_value
                .iter()
                .map(|backtrace| backtrace.label())
                .fold(Label::Bottom, |acc, label| acc.union(label));
            let return_backtrace = LabelBacktrace::new(
                LabelBacktraceKind::FunctionCall,
                context.file(),
                node.location.clone(),
                None,
                label,
                &outcome.return_value,
            )
            .and_then(|backtrace| backtrace.replace_synthetic_tags(&args_labels));

            // If label of arguments has changed, propagate that to the respective symbols
            for (new_arg_backtrace, arg_node) in
                outcome.arguments.clone().into_iter().zip(&node.args)
            {
                let new_arg_backtrace =
                    new_arg_backtrace.and_then(|bt| bt.replace_synthetic_tags(&args_labels));
                if let Some(new_arg_backtrace) = new_arg_backtrace {
                    for (package, name_span) in find_symbols_in_expr(arg_node) {
                        let file = context.file();
                        let package = package_or_current!(context, package);
                        let name = name_span.content();

                        let mut changed = false;
                        if let Some(symbol) = context.symtab_mut().get_symbol_mut(package, name) {
                            changed = matches!(
                                symbol
                                    .backtrace()
                                    .as_ref()
                                    .map(|bt| bt.label())
                                    .unwrap_or(&Label::Bottom)
                                    .partial_cmp(new_arg_backtrace.label()),
                                Some(std::cmp::Ordering::Less) | None
                            );
                            if changed {
                                let new_backtrace = LabelBacktrace::from_children(
                                    symbol
                                        .backtrace()
                                        .iter()
                                        .chain(std::iter::once(&new_arg_backtrace)),
                                    LabelBacktraceKind::FunctionArgumentMutation,
                                    file,
                                    name_span.location(),
                                    Some(name_span),
                                );
                                symbol.set_backtrace(new_backtrace);
                            }
                        }

                        if changed && !context.symtab().is_local(package, name) {
                            context.enqueue_symbol_reverse_dependencies(package, name);
                        }
                    }
                }
            }

            return return_backtrace;
        } else if !is_local {
            context.enqueue_symbol(func_package, name.id.content());
        }
    }

    backtrace
}

pub fn visit_return<'a>(
    context: &mut VisitFileContext<'a, '_>,
    exprs: &[ExprNode<'a>],
    location: &Location,
) {
    let expr_backtraces: Vec<_> = exprs.iter().map(|node| visit_expr(context, node)).collect();
    let branch_backtrace = context.branch_backtrace().cloned();

    let return_backtrace = LabelBacktrace::from_children(
        expr_backtraces.iter().flatten().chain(&branch_backtrace),
        LabelBacktraceKind::Return,
        context.file(),
        location.clone(),
        None,
    );

    if let Some(backtrace) = return_backtrace {
        context.push_return_backtraces(backtrace);
    }
}
