use std::collections::HashMap;

use parser::Span;

use crate::labels::LabelBacktrace;

pub type SymbolScope<'a> = HashMap<&'a str, Symbol<'a>>;

// TODO: review this for multiple file support
#[derive(Debug, Clone)]
pub struct SymbolTable<'a, 'b> {
    parent_scope: Option<&'b SymbolScope<'a>>,
    scopes: Vec<SymbolScope<'a>>,
}

impl<'a, 'b> SymbolTable<'a, 'b> {
    // TODO needed?
    #[allow(dead_code)]
    pub fn new() -> Self {
        SymbolTable {
            parent_scope: None,
            scopes: vec![SymbolScope::new()],
        }
    }

    pub fn new_from_global(parent_scope: &'b SymbolScope<'a>) -> Self {
        SymbolTable {
            parent_scope: Some(parent_scope),
            scopes: vec![SymbolScope::new()],
        }
    }

    // return and replace symbol if already exists
    pub fn create_symbol(&mut self, symbol: Symbol<'a>) -> Option<Symbol<'a>> {
        let scope = self
            .scopes
            .last_mut()
            .expect("symbol table should always have at least one scope");
        scope.insert(symbol.name.content(), symbol)
    }

    pub fn get_symbol<'c>(&'c self, symbol_name: &str) -> Option<&'c Symbol<'a>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|context| context.get(symbol_name))
            .or_else(|| {
                self.parent_scope
                    .and_then(|context| context.get(symbol_name))
            })
    }

    pub fn get_symbol_mut<'c>(&'c mut self, symbol_name: &str) -> Option<&'c mut Symbol<'a>> {
        // FIXME: this does not support globals because cannot get mut ref
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|context| context.get_mut(symbol_name))
    }

    pub fn get_topmost_scope(self) -> SymbolScope<'a> {
        self.scopes
            .into_iter()
            .next()
            .expect("symbol table should always have at least one scope")
    }

    pub fn push(&mut self) {
        self.scopes.push(SymbolScope::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() <= 1 {
            panic!("cannot pop the last symbol scope in the symbol table");
        }
        self.scopes.pop();
    }
}

#[derive(Debug, Clone)]
pub struct Symbol<'a> {
    package: Option<&'a str>, // for qualified operand names
    name: Span<'a>,
    label_backtrace: Option<LabelBacktrace<'a>>,
}

impl<'a> Symbol<'a> {
    pub fn new_with_package(
        package: &'a str,
        name: Span<'a>,
        label_backtrace: Option<LabelBacktrace<'a>>,
    ) -> Symbol<'a> {
        Symbol {
            package: Some(package),
            name,
            label_backtrace,
        }
    }

    pub fn package(&self) -> &Option<&'a str> {
        &self.package
    }

    pub fn name(&self) -> &Span<'a> {
        &self.name
    }

    pub fn backtrace(&self) -> &Option<LabelBacktrace<'a>> {
        &self.label_backtrace
    }

    pub fn set_backtrace(&mut self, backtrace: LabelBacktrace<'a>) {
        self.label_backtrace = Some(backtrace)
    }

    pub fn set_bottom(&mut self) {
        self.label_backtrace = None
    }
}
