use crate::{lexer::LexingError, Span};

pub enum ParsingError<'a> {
    Lexing(LexingError<'a>),
    // other parsing-specific ones...
}

pub struct ErrorDiagnosticInfo<'a> {
    pub code: String,
    pub overview: String,
    pub details: String,
    pub context: Span<'a>,
}

pub trait Diagnostics<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a>;
}

impl<'a> Diagnostics<'a> for ParsingError<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a> {
        match self {
            Self::Lexing(e) => e.diagnostics(),
        }
    }
}
