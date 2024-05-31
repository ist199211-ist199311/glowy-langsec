use std::ops::Range;

use parser::{ParsingError, Span};

use crate::labels::{Label, LabelBacktrace};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ErrorLocation {
    file: usize,
    location: Range<usize>, // location in file (from span)
}

impl ErrorLocation {
    pub fn new(file: usize, location: Range<usize>) -> Self {
        Self { file, location }
    }
}

#[derive(Debug)]
pub enum AnalysisError<'a> {
    // TODO
    Parsing {
        file: usize,
        error: ParsingError<'a>,
    },
    DataFlowAssignment {
        sink_label: Label<'a>,
        label_backtrace: LabelBacktrace<'a>,
    },
    DataFlowFuncCall {
        sink_label: Label<'a>,
        label_backtrace: LabelBacktrace<'a>,
    },
    UnknownSymbol {
        file: usize,
        symbol: Span<'a>,
    },
    Redeclaration {
        file: usize,
        prev_symbol: Span<'a>,
        new_symbol: Span<'a>,
    },
}
