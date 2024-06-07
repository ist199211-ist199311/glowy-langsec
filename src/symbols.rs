use std::collections::HashMap;

use parser::Span;

use crate::labels::LabelBacktrace;

type SymbolId<'a> = (Option<&'a str>, &'a str); // package is None for builtins
type SymbolScope<'a> = HashMap<SymbolId<'a>, Symbol<'a>>;

// TODO: review this for multiple file support
#[derive(Debug, Clone)]
pub struct SymbolTable<'a> {
    scopes: Vec<SymbolScope<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> Self {
        let mut symtab = SymbolTable {
            scopes: vec![SymbolScope::new()],
        };
        for identifier in PREDECLARED_IDENTIFIERS {
            symtab.create_symbol(Symbol::new_predeclared(identifier));
        }
        symtab
    }

    // return and replace symbol if already exists
    pub fn create_symbol(&mut self, symbol: Symbol<'a>) -> Option<Symbol<'a>> {
        let scope = self
            .scopes
            .last_mut()
            .expect("symbol table should always have at least one scope");
        scope.insert((symbol.package, symbol.name.content()), symbol)
    }

    pub fn get_symbol<'b>(
        &'b self,
        package: &'a str,
        symbol_name: &'a str,
    ) -> Option<&'b Symbol<'a>> {
        self.scopes.iter().rev().find_map(|context| {
            context
                .get(&(Some(package), symbol_name))
                .or_else(|| context.get(&(None, symbol_name)))
        })
    }

    pub fn get_symbol_mut<'b>(
        &'b mut self,
        package: &'a str,
        symbol_name: &'a str,
    ) -> Option<&'b mut Symbol<'a>> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|context| context.get_mut(&(Some(package), symbol_name)))
    }

    pub fn is_current_scope(&self, package: &'a str, symbol_name: &'a str) -> bool {
        self.scopes
            .last()
            .iter()
            .any(|scope| scope.contains_key(&(Some(package), symbol_name)))
    }

    // Whether this symbol is in a scope that is not the topmost scope
    pub fn is_local(&self, package: &'a str, symbol_name: &'a str) -> bool {
        self.scopes
            .iter()
            .skip(1)
            .any(|scope| scope.contains_key(&(Some(package), symbol_name)))
    }

    pub fn push(&mut self) {
        self.scopes.push(SymbolScope::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() <= 1 {
            panic!("cannot pop the topmost symbol scope in the symbol table");
        }
        self.scopes.pop();
    }
}

#[derive(Debug, Clone)]
pub struct Symbol<'a> {
    package: Option<&'a str>, // for qualified operand names
    name: Span<'a>,
    label_backtrace: Option<LabelBacktrace<'a>>,
    mutable: bool,
}

impl<'a> Symbol<'a> {
    pub fn new_with_package(
        package: &'a str,
        name: Span<'a>,
        label_backtrace: Option<LabelBacktrace<'a>>,
        mutable: bool,
    ) -> Symbol<'a> {
        Symbol {
            package: Some(package),
            name,
            label_backtrace,
            mutable,
        }
    }

    pub fn new_predeclared(name: &'a str) -> Symbol<'a> {
        Symbol {
            package: None,
            name: Span::new(name, 0, 0), // FIXME: predeclared identifiers don't have a span
            label_backtrace: None,
            mutable: false,
        }
    }

    pub fn package(&self) -> Option<&'a str> {
        self.package
    }

    pub fn name(&self) -> &Span<'a> {
        &self.name
    }

    pub fn backtrace(&self) -> &Option<LabelBacktrace<'a>> {
        &self.label_backtrace
    }

    pub fn set_backtrace(&mut self, backtrace: Option<LabelBacktrace<'a>>) {
        self.label_backtrace = backtrace
    }

    pub fn set_bottom(&mut self) {
        self.label_backtrace = None
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }
}

// https://go.dev/ref/spec#Predeclared_identifiers
const PREDECLARED_IDENTIFIERS: &[&str] = &[
    // Types
    "any",
    "bool",
    "byte",
    "comparable",
    "complex64",
    "complex128",
    "error",
    "float32",
    "float64",
    "int",
    "int8",
    "int16",
    "int32",
    "int64",
    "rune",
    "string",
    "uint",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "uintptr",
    // Constants
    "true",
    "false",
    "iota",
    // Zero value
    "nil",
    // Functions
    "append",
    "cap",
    "clear",
    "close",
    "complex",
    "copy",
    "delete",
    "imag",
    "len",
    "make",
    "max",
    "min",
    "new",
    "panic",
    "print",
    "println",
    "real",
    "recover",
];
