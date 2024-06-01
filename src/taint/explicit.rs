use std::cmp::Ordering;

use parser::{
    ast::{AssignmentNode, BindingDeclSpecNode, ShortVarDeclNode},
    Annotation,
};

use super::visit_expr;
use crate::{
    context::VisitFileContext,
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceType},
    symbols::Symbol,
};

pub fn visit_binding_decl_spec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &BindingDeclSpecNode<'a>,
    annotation: &Option<Box<Annotation<'a>>>,
) {
    for (name, expr) in &node.mapping {
        let expr_backtrace = visit_expr(context, expr);
        let mut label = expr_backtrace
            .iter()
            .fold(Label::Bottom, |acc, backtrace| acc.union(backtrace.label()));
        let mut backtraces = vec![];

        if let Some(annotation) = annotation {
            match annotation.scope {
                "label" => {
                    let annotation_label = Label::from_parts(&annotation.labels);
                    label = label.union(&annotation_label);
                    backtraces.push(LabelBacktrace::new_explicit_annotation(
                        context.file(),
                        name.clone(),
                        annotation_label,
                    ));
                }
                "sink" => {
                    let sink_label = Label::from_parts(&annotation.labels);

                    if let None | Some(Ordering::Greater) = label.partial_cmp(&sink_label) {
                        let backtrace = LabelBacktrace::new(
                            LabelBacktraceType::Assignment,
                            context.file(),
                            name.clone(),
                            label.clone(),
                            backtraces.iter().chain(expr_backtrace.iter()),
                        )
                        .expect("assignment label should not be bottom");

                        context.report_error(
                            name.location(),
                            AnalysisError::InsecureFlow {
                                kind: InsecureFlowKind::Assignment,
                                sink_label,
                                backtrace,
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        let backtrace = LabelBacktrace::new(
            LabelBacktraceType::Assignment,
            context.file(),
            name.clone(),
            label,
            backtraces.iter().chain(expr_backtrace.iter()),
        );

        let new_symbol =
            Symbol::new_with_package(context.current_package(), name.clone(), backtrace);
        if let Some(prev_symbol) = context.symbol_table.create_symbol(new_symbol) {
            context.report_error(
                name.location(),
                AnalysisError::Redeclaration {
                    file: context.file(),
                    prev_symbol: prev_symbol.name().clone(),
                    new_symbol: name.clone(),
                },
            )
        }
    }
}

pub fn visit_short_var_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &ShortVarDeclNode<'a>,
) {
    // possibly merge with visit_binding_decl_spec
    todo!()
}

pub fn visit_assignment<'a>(context: &mut VisitFileContext<'a, '_>, node: &AssignmentNode<'a>) {
    // possibly merge with part of visit_binding_decl_spec
    todo!()
}
