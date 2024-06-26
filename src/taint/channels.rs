use std::cmp::Ordering;

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
    errors::{AnalysisError, InsecureFlowKind},
    labels::{Label, LabelBacktrace, LabelBacktraceKind},
};

// this cannot be a function because borrow checker :(
macro_rules! get_channel_symbol {
    ($context:expr, $expr:expr, $default_location:expr) => {
        get_channel_symbol!($context, $expr, $default_location, ())
    };
    ($context:expr, $expr:expr, $default_location:expr, $err_ret:expr) => {
        if let ExprNode::Name(name) = $expr {
            let package = package_or_current!($context, name.package);
            if !$context.symtab().is_local(package, name.id.content()) {
                if let Some(current_symbol) = $context.current_symbol() {
                    $context.add_symbol_reverse_dependency(
                        ($context.current_package(), current_symbol),
                        (package, name.id.content()),
                    );
                }
            }
            let opt = $context
                .symtab_mut()
                .get_symbol_mut(package, name.id.content());

            if let Some(symbol) = opt {
                symbol
            } else {
                $context.report_error(AnalysisError::UnknownSymbol {
                    file: $context.file(),
                    symbol: name.id.clone(),
                });

                return $err_ret;
            }
        } else {
            let loc = find_expr_location($expr).unwrap_or_else(|| $default_location.clone());

            $context.report_error(AnalysisError::UnsupportedChannelExpr {
                file: $context.file(),
                location: loc,
            });

            return $err_ret;
        }
    };
}

pub fn visit_send<'a>(context: &mut VisitFileContext<'a, '_>, node: &SendNode<'a>) {
    let expr_backtrace = if let Some(backtrace) = visit_expr(context, &node.expr) {
        backtrace
    } else {
        return; // expression is bottom, so we don't need to do anything
    };
    let branch_backtrace = context.branch_backtrace().cloned();

    let file = context.file();

    let symbol = get_channel_symbol!(context, &node.channel, node.location);

    let backtrace = LabelBacktrace::from_children(
        [&branch_backtrace, symbol.backtrace()]
            .into_iter()
            .flatten()
            .chain(std::iter::once(&expr_backtrace)),
        LabelBacktraceKind::Send,
        file,
        node.location.clone(),
        Some(symbol.name().clone()),
    )
    .unwrap(); // safe since at least expr_backtrace is not Bottom

    symbol.set_backtrace(Some(backtrace.clone()));

    if let Some(annotation) = &node.annotation {
        if annotation.scope == "sink" {
            let sink_label = Label::from_parts(&annotation.tags);

            if let None | Some(Ordering::Greater) = backtrace.label().partial_cmp(&sink_label) {
                context.report_error(AnalysisError::InsecureFlow {
                    kind: InsecureFlowKind::Send,
                    sink_label,
                    backtrace,
                });
            }
        }
        // TODO: else { error, invalid scope }
    }
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
