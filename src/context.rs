use std::{collections::HashMap, ops::Range};

use crate::{
    errors::{AnalysisError, ErrorLocation},
    labels::LabelBacktrace,
    symbols::{SymbolScope, SymbolTable},
};

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    /// Read-only view of the global scope (variables).
    /// Updated with the context in the symbol table after visiting
    /// each file.
    pub global_scope: SymbolScope<'a>,
    #[allow(dead_code)]
    /// Map of ((package, name), function)
    functions: HashMap<(&'a str, &'a str), FunctionContext>,
    /// Map of error location and the respective error.
    /// This is a map to avoid reporting the same error multiple times.
    pub errors: HashMap<ErrorLocation, AnalysisError<'a>>,
}

impl<'a> AnalysisContext<'a> {
    pub fn new() -> Self {
        Self {
            global_scope: SymbolScope::new(),
            functions: HashMap::new(),
            errors: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionContext {
    // TODO
    // outcomes: HashMap<parameters_labels, parameters_labels + result_labels>
}

#[derive(Debug)]
pub struct VisitFileContext<'a, 'b> {
    analysis_context: &'b AnalysisContext<'a>,
    file_id: usize,
    pub symbol_table: SymbolTable<'a, 'b>,
    current_package: &'a str,
    pub errors: HashMap<ErrorLocation, AnalysisError<'a>>,
    branch_labels: Vec<LabelBacktrace<'a>>, // stack, for implicit flows
}

impl<'a, 'b> VisitFileContext<'a, 'b> {
    pub fn new(
        analysis_context: &'b AnalysisContext<'a>,
        file_id: usize,
        package: &'a str,
    ) -> Self {
        VisitFileContext {
            analysis_context,
            file_id,
            symbol_table: SymbolTable::new_from_global(&analysis_context.global_scope),
            current_package: package,
            errors: HashMap::new(),
            branch_labels: vec![],
        }
    }

    pub fn report_error(&mut self, location: Range<usize>, error: AnalysisError<'a>) {
        let error_location = ErrorLocation::new(self.file_id, location);

        if !self.analysis_context.errors.contains_key(&error_location)
            && !self.errors.contains_key(&error_location)
        {
            self.errors.insert(error_location, error);
        }
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn current_package(&self) -> &'a str {
        self.current_package
    }

    pub fn symtab(&self) -> &SymbolTable<'a, 'b> {
        &self.symbol_table
    }

    pub fn branch_backtrace(&self) -> Option<&LabelBacktrace<'a>> {
        self.branch_labels.last()
    }

    pub fn push_branch_label(&mut self, backtrace: LabelBacktrace<'a>) {
        // merge with existing branch label
        let composite = if let Some(existing) = self.branch_backtrace() {
            backtrace.with_child(existing)
        } else {
            backtrace
        };

        self.branch_labels.push(composite);
    }

    pub fn pop_branch_label(&mut self) {
        self.branch_labels.pop();
    }
}
