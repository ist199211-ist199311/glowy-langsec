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
    // Go errors
    Parsing {
        file: usize,
        error: ParsingError<'a>,
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

    // IFC errors
    InsecureFlow {
        kind: InsecureFlowKind,
        sink_label: Label<'a>,
        backtrace: LabelBacktrace<'a>,
    },
}

#[derive(Debug)]
pub enum InsecureFlowKind {
    Assignment,
    Call,
}

impl InsecureFlowKind {
    pub fn code(&self) -> u8 {
        match self {
            Self::Assignment => 1,
            Self::Call => 2,
        }
    }

    pub fn context(&self) -> &'static str {
        match self {
            Self::Assignment => "assignment",
            Self::Call => "function call",
        }
    }

    pub fn operand(&self) -> &'static str {
        match self {
            Self::Assignment => "the expression being assigned",
            Self::Call => "an argument",
        }
    }
}
