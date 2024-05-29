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
    file_id: usize,
    node: &SourceFileNode<'a>,
) -> bool {
    let package = node.package_clause.id.content();

    let mut changed = false;

    let mut visit_context = VisitFileContext::new(context, file_id, package);

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
        DeclNode::Function(func_node) => {
            visit_function_decl(context, func_node);
        }
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

fn visit_function_decl<'a>(context: &mut VisitFileContext<'a, '_>, node: &FunctionDeclNode<'a>) {
    context.symbol_table.push();

    for param in &node.signature.params {
        for id in &param.ids {
            if context.symbol_table.create_symbol(Symbol::new_with_package(
                context.current_package(),
                id.content(),
                Label::Bottom, // TODO: make label depend on calls to function
            )) {
                context.report_error(id.location(), AnalysisError::Redeclaration)
            }
        }
    }

    for statements in &node.body {
        visit_statement(context, statements);
    }

    context.symbol_table.pop();
}

fn visit_statement<'a>(context: &mut VisitFileContext<'a, '_>, node: &StatementNode<'a>) {
    match node {
        StatementNode::Empty => {}
        StatementNode::Expr(expr_node) => {
            visit_expr(context, expr_node);
        }
        StatementNode::Send(_) => todo!(),
        StatementNode::Inc(_) | StatementNode::Dec(_) => todo!(),
        StatementNode::Assignment(_) => todo!(),
        StatementNode::ShortVarDecl(_) => todo!(),
        StatementNode::Decl(decl_node) => {
            visit_decl(context, decl_node);
        }
        StatementNode::If(_) => todo!(),
        StatementNode::Block(_) => todo!(),
        StatementNode::Return(_) => todo!(),
        StatementNode::Go(_) => todo!(),
    }
}

fn visit_expr<'a>(context: &mut VisitFileContext<'a, '_>, node: &ExprNode<'a>) -> Label<'a> {
    match node {
        ExprNode::Name(name) => match context.symtab().get_symbol_label(name.id.content()) {
            Some(label) => label.clone(),
            None => {
                context.report_error(
                    name.id.location(),
                    AnalysisError::UnknownSymbol {
                        file: context.file(),
                        symbol: name.id.clone(),
                    },
                );
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
        ExprNode::Call(call_node) => visit_call(context, call_node),
        ExprNode::Indexing(_) => todo!(),
    }
}

fn visit_call<'a>(context: &mut VisitFileContext<'a, '_>, node: &CallNode<'a>) -> Label<'a> {
    // TODO: use `node.func` to more accurately determine label

    let mut label = Label::Bottom;
    for arg in &node.args {
        let arg_label = visit_expr(context, arg);

        if let Some(annotation) = &node.annotation {
            if annotation.scope == "sink" {
                let sink_label = Label::from_parts(&annotation.labels);
                match arg_label.partial_cmp(&sink_label) {
                    None | Some(Ordering::Greater) => {
                        // TODO: FIXME: parser does not have a way to get the location of args
                        context.report_error(0..0, AnalysisError::DataFlow)
                    }
                    _ => {}
                }
            }
        }

        label = label.union(&arg_label);
    }

    label
}
