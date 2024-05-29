use std::ops::Range;

use parser::ParsingError;

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
    DataFlow,
    UnknownSymbol,
    Redeclaration,
}
