use parser::{Location, ParsingError, Span};

use crate::labels::{Label, LabelBacktrace};

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
    MultiComplexAssignment {
        file: usize,
        location: Location,
        num: usize,
    },
    UnevenAssignment {
        file: usize,
        location: Location,
        left: usize,
        right: usize,
    },
    InvalidLeftValue {
        file: usize,
        location: Location,
    },
    ImmutableLeftValue {
        file: usize,
        symbol: Span<'a>,
    },
    UnevenShortVarDecl {
        file: usize,
        location: Location,
        left: usize,
        right: usize,
    },
    GoNotCall {
        file: usize,
        location: Location,
    },
    UnsupportedChannelExpr {
        file: usize,
        location: Location,
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
    Send,
}

impl InsecureFlowKind {
    pub fn code(&self) -> u8 {
        match self {
            Self::Assignment => 1,
            Self::Call => 2,
            Self::Send => 3,
        }
    }

    pub fn context(&self) -> &'static str {
        match self {
            Self::Assignment => "assignment",
            Self::Call => "function call",
            Self::Send => "send statement",
        }
    }

    pub fn operand(&self) -> &'static str {
        match self {
            Self::Assignment => "the expression being assigned",
            Self::Call => "argument",
            Self::Send => "the expression being sent",
        }
    }
}
