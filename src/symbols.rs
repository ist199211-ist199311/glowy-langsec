use std::collections::HashMap;

use crate::labels::Label;

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

    // return true if already exists
    pub fn create_symbol(&mut self, symbol: Symbol<'a>) -> bool {
        let scope = self
            .scopes
            .last_mut()
            .expect("symbol table should always have at least one scope");
        if scope.get(symbol.name).is_some() {
            return true;
        }
        scope.insert(symbol.name, symbol);
        false
    }

    pub fn get_symbol_label<'c>(&'c self, symbol_name: &str) -> Option<&'c Label<'a>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|context| context.get(symbol_name))
            .or_else(|| {
                self.parent_scope
                    .and_then(|context| context.get(symbol_name))
            })
            .map(|symbol| &symbol.label)
    }

    pub fn get_topmost_scope(self) -> SymbolScope<'a> {
        self.scopes
            .into_iter()
            .next()
            .expect("symbol table should always have at least one scope")
    }
}

#[derive(Debug, Clone)]
pub struct Symbol<'a> {
    package: Option<&'a str>, // for qualified operand names
    name: &'a str,
    label: Label<'a>,
}

impl<'a> Symbol<'a> {
    pub fn new_with_package(package: &'a str, name: &'a str, label: Label<'a>) -> Symbol<'a> {
        Symbol {
            package: Some(package),
            name,
            label,
        }
    }

    // TODO needed?
    #[allow(dead_code)]
    pub fn new(name: &'a str, label: Label<'a>) -> Symbol<'a> {
        Symbol {
            package: None,
            name,
            label,
        }
    }
}
