use std::ops::Range;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ErrorLocation<'a> {
    file: &'a str,
    location: Range<usize>, // location in file (from span)
}

impl<'a> ErrorLocation<'a> {
    pub fn new(file: &'a str, location: Range<usize>) -> Self {
        Self { file, location }
    }
}

#[derive(Debug)]
pub enum AnalysisError {
    // TODO
    DataFlow,
    UnknownSymbol,
    Redeclaration,
}
