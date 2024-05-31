use std::cmp::Ordering;

use parser::{ast::*, Annotation, Span};

use crate::{
    context::{AnalysisContext, VisitFileContext},
    errors::AnalysisError,
    labels::{Label, LabelBacktrace, LabelBacktraceType},
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

fn visit_function_decl<'a>(context: &mut VisitFileContext<'a, '_>, node: &FunctionDeclNode<'a>) {
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

fn visit_expr<'a>(
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
        ExprNode::Call(call_node) => visit_call(context, call_node),
        ExprNode::Indexing(_) => todo!(),
    }
}

fn visit_call<'a>(
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

fn find_first_ident<'a>(node: &ExprNode<'a>) -> Option<Span<'a>> {
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
