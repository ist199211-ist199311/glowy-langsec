use std::{iter::Peekable, ops::Range};

use ast::SourceFileNode;
pub use errors::{Diagnostics, ErrorDiagnosticInfo, ParsingError};
use lexer::Lexer;

use crate::parser::parse_source_file;

pub mod ast;
mod errors;
mod lexer;
mod parser;
mod token;

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

    pub fn location(&self) -> Range<usize> {
        self.offset..(self.offset + self.content.len())
    }
}

type TokenStream<'a> = Peekable<Lexer<'a>>;

pub fn parse(input: &str) -> Result<SourceFileNode, ParsingError> {
    let mut stream = Lexer::new(input).peekable();

    parse_source_file(&mut stream)
}
