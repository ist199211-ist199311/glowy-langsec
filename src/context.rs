use std::collections::{HashMap, HashSet};

use crate::{
    errors::AnalysisError,
    labels::LabelBacktrace,
    symbols::{Symbol, SymbolTable},
};

type SymbolId<'a> = (&'a str, &'a str);

#[derive(Debug)]
pub struct AnalysisContext<'a> {
    /// Symbols of the entire program, where the topmost scope represents
    /// the global scope.
    symbol_table: SymbolTable<'a>,
    /// Queue of symbols to visit
    symbol_queue: HashSet<SymbolId<'a>>,
    /// Map of metadata for top-level functions
    functions: HashMap<SymbolId<'a>, FunctionMetadata<'a>>,
    /// Map of which symbols (values) depend on a certain symbol (key)
    reverse_dependencies: HashMap<SymbolId<'a>, HashSet<SymbolId<'a>>>,
    /// Whether the analysis is in a stage that errors can be emitted
    accept_errors: bool,
    /// Errors emitted during analysis
    errors: Vec<AnalysisError<'a>>,
}

impl<'a> AnalysisContext<'a> {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            symbol_queue: HashSet::new(),
            functions: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            accept_errors: true,
            errors: Vec::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.symbol_queue.is_empty()
    }

    pub fn enable_errors(&mut self) {
        self.accept_errors = true
    }

    pub fn disable_errors(&mut self) {
        self.accept_errors = false
    }

    pub fn into_errors(self) -> Vec<AnalysisError<'a>> {
        self.errors
    }
}

/// Extra metadata about top-level functions
#[derive(Debug, Default)]
struct FunctionMetadata<'a> {
    outcome: Option<FunctionOutcome<'a>>,
}

/// Represents how the arguments of a function affect the label of its result
/// value, as well as its arguments when they are mutable (e.g., channels)
#[derive(Debug, PartialEq)]
pub struct FunctionOutcome<'a> {
    /// New labels of arguments (if applicable)
    pub arguments: Vec<Option<LabelBacktrace<'a>>>,
    /// Label of return values. It's a Vec, since there can be multiple return
    /// statements. It is responsibility of the caller to generate a label
    /// backtrace from this.
    pub return_value: Vec<LabelBacktrace<'a>>,
}

#[derive(Debug)]
pub struct VisitFileContext<'a, 'b> {
    analysis_context: &'b mut AnalysisContext<'a>,
    file_id: usize,
    current_package: &'a str,
    current_symbol: Option<&'a str>, // name of the top-level symbol being visited
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
            current_symbol: None,
            branch_backtraces: vec![],
            return_backtraces: vec![],
        }
    }

    pub fn report_error(&mut self, error: AnalysisError<'a>) {
        if self.analysis_context.accept_errors {
            self.analysis_context.errors.push(error);
        }
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn current_package(&self) -> &'a str {
        self.current_package
    }

    pub fn current_symbol(&self) -> Option<&'a str> {
        self.current_symbol
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

    /// Returns whether the outcome has changed
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

    /// Enqueue all symbols that depend on the given symbol.
    /// To be used to propagate labels when the label of a symbol changes.
    pub fn enqueue_symbol_reverse_dependencies(&mut self, package: &'a str, name: &'a str) {
        // Only enqueue symbols that we can visit
        if let Some(dependencies) = self
            .analysis_context
            .reverse_dependencies
            .get(&(package, name))
        {
            dependencies
                .iter()
                // Ensure that only symbols that can be visited are enqueued
                .filter(|dependency| {
                    self.analysis_context
                        .reverse_dependencies
                        .contains_key(dependency)
                })
                .for_each(|dependency| {
                    self.analysis_context.symbol_queue.insert(*dependency);
                })
        }
    }

    pub fn enqueue_symbol(&mut self, package: &'a str, name: &'a str) {
        // Only enqueue symbols that are known globals.
        // This avoids accidentally enqueuing functions/decls that the analyzer will not
        // visit, preventing infinite loops.
        if self
            .analysis_context
            .reverse_dependencies
            .contains_key(&(package, name))
        {
            self.analysis_context.symbol_queue.insert((package, name));
        }
    }

    pub fn add_symbol_reverse_dependency(
        &mut self,
        from: (&'a str, &'a str),
        to: (&'a str, &'a str),
    ) {
        // Only symbols that already have an entry can have reverse dependencies added
        // to them. This avoids accidentally enqueuing functions/decls that the
        // analyzer will not visit, preventing infinite loops.
        if let Some(dependencies) = self.analysis_context.reverse_dependencies.get_mut(&to) {
            dependencies.insert(from);
        }
    }

    pub fn should_visit_global_symbol(&self, package: &'a str, name: &'a str) -> bool {
        // when errors are enabled, we should visit everything
        self.analysis_context.accept_errors
            || self
                .analysis_context
                .symbol_queue
                .contains(&(package, name))
    }

    pub fn enter_global_symbol(&mut self, package: &'a str, name: &'a str) {
        self.current_symbol = Some(name);
        self.analysis_context.symbol_queue.remove(&(package, name));
    }

    pub fn leave_global_symbol(&mut self) {
        self.current_symbol = None;
    }

    pub fn is_in_global_symbol(&self) -> bool {
        self.current_symbol.is_some()
    }

    pub fn declare_global_symbol(&mut self, symbol: Symbol<'a>) -> Option<Symbol<'a>> {
        if let Some(package) = symbol.package() {
            self.analysis_context
                .reverse_dependencies
                .insert((package, symbol.name().content()), HashSet::new());
            self.analysis_context
                .symbol_queue
                .insert((package, symbol.name().content()));
        }
        self.symtab_mut().create_symbol(symbol)
    }
}
