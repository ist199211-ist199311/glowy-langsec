use std::cmp::Ordering;

use parser::{
    ast::{AssignmentNode, BindingDeclSpecNode, ShortVarDeclNode},
    Annotation,
};

use super::visit_expr;
use crate::{
    context::VisitFileContext,
    errors::AnalysisError,
    labels::{Label, LabelBacktrace, LabelBacktraceType},
    symbols::Symbol,
};

pub fn visit_binding_decl_spec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &BindingDeclSpecNode<'a>,
    annotation: &Option<Box<Annotation<'a>>>,
) {
    for (name, expr) in &node.mapping {
        let assignment_label_backtrace = visit_expr(context, expr);
        let mut label = assignment_label_backtrace
            .iter()
            .fold(Label::Bottom, |acc, backtrace| acc.union(backtrace.label()));
        let mut label_backtraces = vec![];

        if let Some(annotation) = annotation {
            match annotation.scope {
                "label" => {
                    let annotation_label = Label::from_parts(&annotation.labels);
                    label = label.union(&annotation_label);
                    label_backtraces.push(LabelBacktrace::new_explicit_annotation(
                        context.file(),
                        name.clone(),
                        annotation_label,
                    ));
                }
                "sink" => {
                    let sink_label = Label::from_parts(&annotation.labels);
                    match label.partial_cmp(&sink_label) {
                        None | Some(Ordering::Greater) => {
                            let label_backtrace = LabelBacktrace::new(
                                LabelBacktraceType::Assignment,
                                context.file(),
                                name.clone(),
                                label.clone(),
                                label_backtraces
                                    .iter()
                                    .chain(assignment_label_backtrace.iter()),
                            );
                            context.report_error(
                                name.location(),
                                AnalysisError::DataFlowAssignment {
                                    sink_label,
                                    label_backtrace: label_backtrace
                                        .expect("label of assignment to not be bottom"),
                                },
                            )
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let label_backtrace = LabelBacktrace::new(
            LabelBacktraceType::Assignment,
            context.file(),
            name.clone(),
            label,
            label_backtraces
                .iter()
                .chain(assignment_label_backtrace.iter()),
        );

        let new_symbol =
            Symbol::new_with_package(context.current_package(), name.clone(), label_backtrace);
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
