use std::{collections::HashMap, ops::Range};

use crate::{
    errors::{AnalysisError, ErrorLocation},
    labels::LabelBacktrace,
    symbols::SymbolTable,
};

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    /// Symbols of the entire program, where the topmost scope represents
    /// the global scope.
    symbol_table: SymbolTable<'a>,
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
            symbol_table: SymbolTable::new(),
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
    analysis_context: &'b mut AnalysisContext<'a>,
    file_id: usize,
    current_package: &'a str,
    branch_backtraces: Vec<LabelBacktrace<'a>>, // stack, for implicit flows
}

impl<'a, 'b> VisitFileContext<'a, 'b> {
    pub fn new(
        analysis_context: &'b mut AnalysisContext<'a>,
        file_id: usize,
        package: &'a str,
    ) -> Self {
        VisitFileContext {
            analysis_context,
            file_id,
            current_package: package,
            branch_backtraces: vec![],
        }
    }

    pub fn report_error(&mut self, location: Range<usize>, error: AnalysisError<'a>) {
        let error_location = ErrorLocation::new(self.file_id, location);

        self.analysis_context
            .errors
            .entry(error_location)
            .or_insert(error);
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn current_package(&self) -> &'a str {
        self.current_package
    }

    pub fn symtab(&self) -> &SymbolTable<'a> {
        &self.analysis_context.symbol_table
    }

    pub fn symtab_mut(&mut self) -> &mut SymbolTable<'a> {
        &mut self.analysis_context.symbol_table
    }

    pub fn branch_backtrace(&self) -> Option<&LabelBacktrace<'a>> {
        self.branch_backtraces.last()
    }

    pub fn push_branch_label(&mut self, backtrace: LabelBacktrace<'a>) {
        // merge with existing branch label
        let composite = if let Some(existing) = self.branch_backtrace() {
            backtrace.with_child(existing)
        } else {
            backtrace
        };

        self.branch_backtraces.push(composite);
    }

    pub fn pop_branch_label(&mut self) {
        self.branch_backtraces.pop();
    }
}
