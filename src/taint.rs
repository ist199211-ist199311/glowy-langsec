use explicit::{visit_assignment, visit_binding_decl, visit_short_var_decl};
use exprs::visit_expr;
use funcs::{visit_function_decl, visit_return};
use implicit::{visit_go, visit_if, visit_incdec};
use parser::ast::{DeclNode, ExprNode, SourceFileNode, StatementNode};

use crate::context::{AnalysisContext, VisitFileContext};

mod explicit;
mod exprs;
mod funcs;
mod implicit;

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
        DeclNode::Const {
            specs, annotation, ..
        } => visit_binding_decl(context, specs, false, annotation),
        DeclNode::Var {
            specs, annotation, ..
        } => visit_binding_decl(context, specs, true, annotation),
        DeclNode::Function(func_node) => {
            visit_function_decl(context, func_node);
        }
    }
}

fn visit_statement<'a>(context: &mut VisitFileContext<'a, '_>, node: &StatementNode<'a>) {
    match node {
        StatementNode::Empty => {}
        StatementNode::Expr(expr) => {
            visit_expr(context, expr);
        }
        StatementNode::Send(_) => todo!(),
        StatementNode::Inc { operand, location } | StatementNode::Dec { operand, location } => {
            visit_incdec(context, operand)
        }
        StatementNode::Assignment(assignment) => visit_assignment(context, assignment),
        StatementNode::ShortVarDecl(decl) => visit_short_var_decl(context, decl),
        StatementNode::Decl(decl) => visit_decl(context, decl),
        StatementNode::If(r#if) => visit_if(context, r#if),
        StatementNode::Block(stmts) => {
            context.symtab_mut().push();
            for statement in stmts {
                visit_statement(context, statement);
            }
            context.symtab_mut().pop();
        }
        StatementNode::Return { exprs, location } => visit_return(context, exprs),
        StatementNode::Go(expr) => match expr {
            ExprNode::Call(call) => visit_go(context, call),
            _ => panic!("invalid go statement; expected function call"), // TODO: don't panic
        },
    }
}
