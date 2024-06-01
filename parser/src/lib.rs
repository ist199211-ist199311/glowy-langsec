use std::ops::Range;

use ast::SourceFileNode;
pub use errors::{Diagnostics, ErrorDiagnosticInfo, ParsingError};
use lexer::Lexer;
use stream::TokenStream;
pub use token::Annotation;

use crate::parser::parse_source_file;

pub mod ast;
mod errors;
mod lexer;
mod parser;
mod stream;
mod token;

// this should be scoped by file, or only used in contexts
// where the file referred to is obvious
pub type Location = Range<usize>;

#[derive(Clone, Debug, PartialEq)]
pub struct Span<'a> {
    content: &'a str,
    offset: usize,
    line: usize,
}

impl<'a> Span<'a> {
    pub fn new(content: &'a str, offset: usize, line: usize) -> Self {
        Self {
            content,
            offset,
            line,
        }
    }

    pub fn content(&self) -> &'a str {
        self.content
    }

    pub fn location(&self) -> Range<usize> {
        self.offset..(self.offset + self.content.len())
    }
}

pub fn parse(input: &str) -> Result<SourceFileNode, ParsingError> {
    let mut stream = TokenStream::new(Lexer::new(input));

    parse_source_file(&mut stream)
}
