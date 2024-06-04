use parser::{
    ast::{ExprNode, SendNode},
    Location,
};

use super::{
    exprs::{find_expr_location, visit_expr},
    package_or_current,
};
use crate::{
    context::VisitFileContext,
    errors::AnalysisError,
    labels::{LabelBacktrace, LabelBacktraceKind},
};

// this cannot be a function because borrow checker :(
macro_rules! get_channel_symbol {
    ($context:expr, $expr:expr, $default_location:expr) => {
        get_channel_symbol!($context, $expr, $default_location, ())
    };
    ($context:expr, $expr:expr, $default_location:expr, $err_ret:expr) => {
        if let ExprNode::Name(name) = $expr {
            let package = package_or_current!($context, name.package);
            let opt = $context
                .symtab_mut()
                .get_symbol_mut(package, name.id.content());

            if let Some(symbol) = opt {
                symbol
            } else {
                $context.report_error(
                    name.id.location().clone(),
                    AnalysisError::UnknownSymbol {
                        file: $context.file(),
                        symbol: name.id.clone(),
                    },
                );

                return $err_ret;
            }
        } else {
            let loc = find_expr_location($expr).unwrap_or_else(|| $default_location.clone());

            $context.report_error(
                loc.clone(),
                AnalysisError::UnsupportedChannelExpr {
                    file: $context.file(),
                    location: loc,
                },
            );

            return $err_ret;
        }
    };
}

pub fn visit_send<'a>(context: &mut VisitFileContext<'a, '_>, node: &SendNode<'a>) {
    // TODO: support (label? +) sink annotations

    let expr_backtrace = if let Some(backtrace) = visit_expr(context, &node.expr) {
        backtrace
    } else {
        return; // expression is bottom, so we don't need to do anything
    };
    let branch_backtrace = context.branch_backtrace().cloned();

    let file = context.file();

    let symbol = get_channel_symbol!(context, &node.channel, node.location);

    let backtrace = LabelBacktrace::from_children(
        [&Some(expr_backtrace), &branch_backtrace, symbol.backtrace()].into_iter(),
        LabelBacktraceKind::Send,
        file,
        node.location.clone(),
        Some(symbol.name().clone()),
    )
    .unwrap(); // safe since at least expr_backtrace is not Bottom

    symbol.set_backtrace(backtrace);
}

pub fn visit_receive<'a>(
    context: &mut VisitFileContext<'a, '_>,
    operand: &ExprNode<'a>,
    location: &Location,
) -> Option<LabelBacktrace<'a>> {
    let file = context.file();

    let symbol = get_channel_symbol!(context, operand, location, None);

    if let Some(backtrace) = symbol.backtrace() {
        LabelBacktrace::new(
            LabelBacktraceKind::Receive,
            file,
            location.clone(),
            Some(symbol.name().clone()),
            backtrace.label().clone(),
            std::iter::once(backtrace),
        )
    } else {
        None // bottom
    }
}
