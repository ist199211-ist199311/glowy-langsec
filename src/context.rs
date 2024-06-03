use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

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
    /// Queue of functions to visit
    function_queue: HashSet<(&'a str, &'a str)>,
    /// Map of ((package, name), function)
    pub functions: HashMap<(&'a str, &'a str), FunctionContext<'a>>,
    /// Map of error location and the respective error.
    /// This is a map to avoid reporting the same error multiple times.
    pub errors: HashMap<ErrorLocation, AnalysisError<'a>>,
}

impl<'a> AnalysisContext<'a> {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            function_queue: HashSet::from([("main", "main")]),
            functions: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.function_queue.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct FunctionContext<'a> {
    outcome: Option<FunctionOutcome<'a>>,
    /// Functions that depend on this one
    reverse_dependencies: HashSet<(&'a str, &'a str)>, // (package, function name)
}

#[derive(Debug, PartialEq)]
pub struct FunctionOutcome<'a> {
    pub arguments: Vec<Option<LabelBacktrace<'a>>>,
    pub return_value: Vec<LabelBacktrace<'a>>,
}

#[derive(Debug)]
pub struct VisitFileContext<'a, 'b> {
    analysis_context: &'b mut AnalysisContext<'a>,
    file_id: usize,
    current_package: &'a str,
    current_function: Option<&'a str>,
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
            current_function: None,
            branch_backtraces: vec![],
            return_backtraces: vec![],
        }
    }

    pub fn report_error(&mut self, location: Range<usize>, error: AnalysisError<'a>) {
        let error_location = ErrorLocation::new(self.file_id, location);

        // TODO this needs to be handled better
        self.analysis_context.errors.insert(error_location, error);
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn current_package(&self) -> &'a str {
        self.current_package
    }

    pub fn current_function(&self) -> Option<&'a str> {
        self.current_function
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
    ) -> Option<&FunctionOutcome<'a>> {
        self.analysis_context
            .functions
            .get(&(package, name))
            .and_then(|func_context| func_context.outcome.as_ref())
    }

    pub fn set_function_outcome(
        &mut self,
        package: &'a str,
        name: &'a str,
        outcome: FunctionOutcome<'a>,
    ) -> bool {
        let prev_outcome = &mut self
            .analysis_context
            .functions
            .entry((package, name))
            .or_default()
            .outcome;
        let new_outcome = Some(outcome);
        let changed = *prev_outcome != new_outcome;
        *prev_outcome = new_outcome;
        changed
    }

    pub fn enqueue_function_reverse_dependencies(&mut self, package: &'a str, name: &'a str) {
        if let Some(func_context) = self.analysis_context.functions.get(&(package, name)) {
            func_context
                .reverse_dependencies
                .iter()
                .for_each(|dependency| {
                    self.analysis_context.function_queue.insert(*dependency);
                });
        }
    }

    pub fn enqueue_function(&mut self, package: &'a str, name: &'a str) {
        self.analysis_context.function_queue.insert((package, name));
    }

    pub fn add_function_reverse_dependency(
        &mut self,
        from: (&'a str, &'a str),
        to: (&'a str, &'a str),
    ) {
        self.analysis_context
            .functions
            .entry(to)
            .or_default()
            .reverse_dependencies
            .insert(from);
    }

    pub fn should_visit_function(&self, package: &'a str, name: &'a str) -> bool {
        self.analysis_context
            .function_queue
            .contains(&(package, name))
    }

    pub fn enter_function(&mut self, package: &'a str, name: &'a str) {
        self.current_function = Some(name);
        self.analysis_context
            .function_queue
            .remove(&(package, name));
    }

    pub fn leave_function(&mut self) {
        self.current_function = None;
    }
}
