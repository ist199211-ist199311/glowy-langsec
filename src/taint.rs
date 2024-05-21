use std::cmp::Ordering;

use parser::{ast::*, Annotation};

use crate::{
    context::{AnalysisContext, VisitFileContext},
    errors::AnalysisError,
    labels::Label,
    symbols::Symbol,
};

pub fn visit_source_file<'a>(
    context: &mut AnalysisContext<'a>,
    file_name: &'a str,
    node: &SourceFileNode<'a>,
) -> bool {
    let package = node.package_clause.id.content();

    let mut changed = false;

    // TODO file name
    let mut visit_context = VisitFileContext::new(context, file_name, package);

    for decl in &node.top_level_decls {
        visit_decl(&mut visit_context, decl);
    }

    // TODO this is a mess, should be moved to the impl of AnalysisContext, but
    // the borrow checker doesn't like that
    let global_symbol_scope = visit_context.symbol_table.get_topmost_scope();
    context.errors.extend(visit_context.errors);
    // TODO: broken; needs to compare labels individually
    // changed |= !global_symbol_context.is_empty();
    context.global_scope.extend(global_symbol_scope);

    changed
}

fn visit_decl<'a>(context: &mut VisitFileContext<'a, '_>, node: &DeclNode<'a>) {
    match node {
        DeclNode::Const { specs, annotation } | DeclNode::Var { specs, annotation } => {
            for spec in specs {
                visit_binding_decl_spec(context, spec, annotation);
            }
        }
        DeclNode::Function(_) => todo!(),
    }
}

fn visit_binding_decl_spec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &BindingDeclSpecNode<'a>,
    annotation: &Option<Box<Annotation<'a>>>,
) {
    for (name, expr) in &node.mapping {
        let mut label = visit_expr(context, expr);

        if let Some(annotation) = annotation {
            match annotation.scope {
                "label" => {
                    label = Label::from_parts(&annotation.labels);
                }
                "sink" => {
                    let sink_label = Label::from_parts(&annotation.labels);
                    match label.partial_cmp(&sink_label) {
                        None | Some(Ordering::Greater) => {
                            context.report_error(name.location(), AnalysisError::DataFlow)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if context.symbol_table.create_symbol(Symbol::new_with_package(
            context.current_package(),
            name.content(),
            label,
        )) {
            context.report_error(name.location(), AnalysisError::Redeclaration)
        }
    }
}

fn visit_expr<'a>(context: &mut VisitFileContext<'a, '_>, node: &ExprNode<'a>) -> Label<'a> {
    match node {
        ExprNode::Name(name) => match context.symtab().get_symbol_label(name.id.content()) {
            Some(label) => label.clone(),
            None => {
                context.report_error(name.id.location(), AnalysisError::UnknownSymbol);
                Label::Bottom
            }
        },
        ExprNode::Literal(_) => Label::Bottom,
        ExprNode::UnaryOp { operand, .. } => visit_expr(context, operand.as_ref()),
        ExprNode::BinaryOp { left, right, .. } => {
            let llabel = visit_expr(context, left.as_ref());
            let rlabel = visit_expr(context, right.as_ref());
            llabel.union(&rlabel)
        }
        ExprNode::Call(_) => todo!(),
        ExprNode::Indexing(_) => todo!(),
    }
}
