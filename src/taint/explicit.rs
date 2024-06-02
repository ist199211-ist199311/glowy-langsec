use std::cmp::Ordering;

use parser::{
    ast::{AssignmentKind, AssignmentNode, BindingDeclSpecNode, ExprNode, ShortVarDeclNode},
    Annotation,
};

use super::exprs::{find_expr_location, visit_expr};
use crate::{
    context::VisitFileContext,
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceType},
    symbols::Symbol,
};

pub fn visit_binding_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    specs: &Vec<BindingDeclSpecNode<'a>>,
    mutable: bool,
    annotation: &Option<Box<Annotation<'a>>>,
) {
    for spec in specs {
        visit_binding_decl_spec(context, spec, mutable, annotation);
    }
}

pub fn visit_binding_decl_spec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &BindingDeclSpecNode<'a>,
    mutable: bool,
    annotation: &Option<Box<Annotation<'a>>>,
) {
    for (name, expr) in &node.mapping {
        let expr_backtrace = visit_expr(context, expr);
        let mut label = expr_backtrace
            .as_ref()
            .map(LabelBacktrace::label)
            .cloned()
            .unwrap_or(Label::Bottom);
        let mut backtraces = vec![];

        let branch_backtrace = context.branch_backtrace().cloned();
        label = label.union(&Label::from(&branch_backtrace));
        if let Some(backtrace) = branch_backtrace {
            backtraces.push(backtrace);
        }

        if let Some(annotation) = annotation {
            match annotation.scope {
                "label" => {
                    let annotation_label = Label::from_parts(&annotation.tags);
                    label = label.union(&annotation_label);
                    backtraces.push(LabelBacktrace::new_explicit_annotation(
                        context.file(),
                        name.clone(),
                        annotation_label,
                    ));
                }
                "sink" => {
                    let sink_label = Label::from_parts(&annotation.tags);

                    if let None | Some(Ordering::Greater) = label.partial_cmp(&sink_label) {
                        let backtrace = LabelBacktrace::new(
                            LabelBacktraceType::Assignment,
                            context.file(),
                            find_expr_location(expr).unwrap(), // guaranteed Some
                            None,
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
            name.location(),
            Some(name.clone()),
            label,
            backtraces.iter().chain(expr_backtrace.iter()),
        );

        let new_symbol =
            Symbol::new_with_package(context.current_package(), name.clone(), backtrace, mutable);
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
    // treated here as syntax sugar for a normal var decl, for simplicity

    if node.ids.len() != node.exprs.len() {
        context.report_error(
            node.location.clone(),
            AnalysisError::UnevenShortVarDecl {
                file: context.file(),
                location: node.location.clone(),
                left: node.ids.len(),
                right: node.exprs.len(),
            },
        );

        return;
    }

    visit_binding_decl_spec(
        context,
        &BindingDeclSpecNode {
            mapping: node
                .ids
                .iter()
                .cloned()
                .zip(node.exprs.iter().cloned())
                .collect(),
            r#type: None,
        },
        true,
        &node.annotation,
    );
}

pub fn visit_assignment<'a>(context: &mut VisitFileContext<'a, '_>, node: &AssignmentNode<'a>) {
    // TODO: possibly merge with part of visit_binding_decl_spec

    if node.kind != AssignmentKind::Simple && node.lhs.len() != 1 {
        context.report_error(
            node.location.clone(),
            AnalysisError::MultiComplexAssignment {
                file: context.file(),
                location: node.location.clone(),
                num: node.lhs.len(),
            },
        );

        return;
    } else if node.lhs.len() != node.rhs.len() {
        context.report_error(
            node.location.clone(),
            AnalysisError::UnevenAssignment {
                file: context.file(),
                location: node.location.clone(),
                left: node.lhs.len(),
                right: node.rhs.len(),
            },
        );

        return;
    }

    for (lhs, rhs) in node.lhs.iter().zip(node.rhs.iter()) {
        // sadly this needs to happen beforehand to make the borrow checker happy,
        // otherwise we would use &mut context while also holding an immutable ref
        // to context (symbol)
        let rhs_backtrace = visit_expr(context, rhs);
        let branch_backtrace = context.branch_backtrace().cloned();
        let file = context.file();

        // TODO: support more kinds of left values, e.g. indexing
        let symbol = if let ExprNode::Name(name) = lhs {
            // TODO: support package
            if let Some(sym) = context.symbol_table.get_symbol_mut(name.id.content()) {
                if sym.mutable() {
                    sym
                } else {
                    context.report_error(
                        name.id.location(),
                        AnalysisError::ImmutableLeftValue {
                            file,
                            symbol: name.id.clone(),
                        },
                    );

                    return;
                }
            } else {
                context.report_error(
                    name.id.location(),
                    AnalysisError::UnknownSymbol {
                        file,
                        symbol: name.id.clone(),
                    },
                );

                return;
            }
        } else {
            let loc = find_expr_location(lhs).unwrap_or_else(|| node.location.clone());

            context.report_error(
                loc.clone(),
                AnalysisError::InvalidLeftValue {
                    file,
                    location: loc,
                },
            );

            return;
        };

        let branch_label = Label::from(&branch_backtrace);
        let current_label = Label::from(symbol.backtrace());
        let rhs_label = Label::from(&rhs_backtrace);

        let label = if node.kind == AssignmentKind::Simple {
            rhs_label
        } else {
            current_label.union(&rhs_label)
        };
        let label = label.union(&branch_label);

        if label == Label::Bottom {
            symbol.set_bottom();
        } else {
            symbol.set_backtrace(
                LabelBacktrace::new(
                    LabelBacktraceType::Assignment,
                    file,
                    node.location.clone(),
                    Some(symbol.name().clone()),
                    label,
                    // constructor will get rid of subsequent children if
                    // the ones before are is enough to cover the label
                    [rhs_backtrace, branch_backtrace, symbol.backtrace().clone()]
                        .iter()
                        .filter_map(Option::as_ref),
                )
                .unwrap(), // safe if original backtrace exists
            );
        }
    }
}
