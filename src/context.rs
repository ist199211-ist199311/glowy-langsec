use std::{collections::HashMap, ops::Range};

use crate::{
    errors::{AnalysisError, ErrorLocation},
    labels::{Label, LabelBacktrace},
    symbols::SymbolTable,
};

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    /// Symbols of the entire program, where the topmost scope represents
    /// the global scope.
    symbol_table: SymbolTable<'a>,
    /// Map of ((package, name), function)
    functions: HashMap<(&'a str, &'a str), FunctionContext<'a>>,
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

#[derive(Debug, Default)]
pub struct FunctionContext<'a> {
    /// Map of (argument labels, (argument final labels, return value label))
    outcomes: HashMap<Vec<Label<'a>>, FunctionOutcome<'a>>,
}

#[derive(Debug)]
pub struct FunctionOutcome<'a> {
    pub arguments: Vec<Option<LabelBacktrace<'a>>>,
    pub return_value: Vec<LabelBacktrace<'a>>,
}

#[derive(Debug)]
pub struct VisitFileContext<'a, 'b> {
    analysis_context: &'b mut AnalysisContext<'a>,
    file_id: usize,
    current_package: &'a str,
    branch_backtraces: Vec<LabelBacktrace<'a>>, // stack, for implicit flows
    return_backtraces: Vec<LabelBacktrace<'a>>, // for function return labels
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
            return_backtraces: vec![],
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

    pub fn push_branch_backtrace(&mut self, backtrace: LabelBacktrace<'a>) {
        // merge with existing branch label
        let composite = if let Some(existing) = self.branch_backtrace() {
            backtrace.with_child(existing)
        } else {
            backtrace
        };

        self.branch_backtraces.push(composite);
    }

    pub fn pop_branch_backtrace(&mut self) {
        self.branch_backtraces.pop();
    }

    pub fn clear_return_backtraces(&mut self) {
        self.return_backtraces.clear();
    }

    pub fn push_return_backtraces(&mut self, backtrace: LabelBacktrace<'a>) {
        self.return_backtraces.push(backtrace);
    }

    pub fn get_return_backtraces(&self) -> &[LabelBacktrace<'a>] {
        &self.return_backtraces
    }

    pub fn get_function_outcome(
        &self,
        package: &'a str,
        name: &'a str,
        arg_labels: &[Label<'a>],
    ) -> Option<&FunctionOutcome<'a>> {
        self.analysis_context
            .functions
            .get(&(package, name))
            .and_then(|func_context| func_context.outcomes.get(arg_labels))
    }

    pub fn set_function_outcome(
        &mut self,
        package: &'a str,
        name: &'a str,
        arg_labels: Vec<Label<'a>>,
        outcome: FunctionOutcome<'a>,
    ) {
        self.analysis_context
            .functions
            .entry((package, name))
            .or_default()
            .outcomes
            .insert(arg_labels, outcome);
    }
}
