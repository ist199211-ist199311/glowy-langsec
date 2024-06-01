use parser::ast::{CallNode, ExprNode, IfNode};

use crate::context::VisitFileContext;

pub fn visit_if<'a>(context: &mut VisitFileContext<'a, '_>, node: &IfNode<'a>) {
    todo!()
}

// TODO: visit_for

pub fn visit_incdec<'a>(context: &mut VisitFileContext<'a, '_>, expr: &ExprNode<'a>) {
    todo!()
}

pub fn visit_go<'a>(context: &mut VisitFileContext<'a, '_>, expr: &CallNode<'a>) {
    todo!()
}
