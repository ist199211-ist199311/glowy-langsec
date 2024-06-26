use parser::{
    ast::{BindingDeclSpecNode, DeclNode, FunctionDeclNode, SourceFileNode},
    Span,
};

use crate::{
    context::{AnalysisContext, VisitFileContext},
    errors::AnalysisError,
    symbols::Symbol,
};

pub fn visit_source_file<'a>(
    context: &mut AnalysisContext<'a>,
    file_id: usize,
    node: &SourceFileNode<'a>,
) {
    let package = node.package_clause.id.content();

    let mut visit_context = VisitFileContext::new(context, file_id, package);

    for decl in &node.top_level_decls {
        visit_decl(&mut visit_context, decl);
    }
}

fn visit_decl<'a>(context: &mut VisitFileContext<'a, '_>, node: &DeclNode<'a>) {
    match node {
        DeclNode::Const { specs, .. } => visit_binding_decl(context, specs, false),
        DeclNode::Var { specs, .. } => visit_binding_decl(context, specs, true),
        DeclNode::Function(func_node) => {
            visit_function_decl(context, func_node);
        }
    }
}

fn visit_binding_decl<'a>(
    context: &mut VisitFileContext<'a, '_>,
    specs: &[BindingDeclSpecNode<'a>],
    mutable: bool,
) {
    for spec in specs {
        visit_binding_decl_spec(context, spec, mutable);
    }
}

fn visit_binding_decl_spec<'a>(
    context: &mut VisitFileContext<'a, '_>,
    node: &BindingDeclSpecNode<'a>,
    mutable: bool,
) {
    for (name, _) in &node.mapping {
        try_declare_new_symbol(context, name, mutable);
    }
}

fn visit_function_decl<'a>(context: &mut VisitFileContext<'a, '_>, node: &FunctionDeclNode<'a>) {
    try_declare_new_symbol(context, &node.name, false);
}

/// Declare a new symbol in the symbol table or emit a warning if it already
/// exists
fn try_declare_new_symbol<'a>(
    context: &mut VisitFileContext<'a, '_>,
    name: &Span<'a>,
    mutable: bool,
) {
    let new_symbol =
        Symbol::new_with_package(context.current_package(), name.clone(), None, mutable);
    if let Some(prev_symbol) = context.declare_global_symbol(new_symbol) {
        context.report_error(AnalysisError::Redeclaration {
            file: context.file(),
            prev_symbol: prev_symbol.name().clone(),
            new_symbol: name.clone(),
        })
    }
}
