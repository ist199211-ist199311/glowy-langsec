use std::str::Chars;

use finl_unicode::categories::CharacterCategories;

use crate::{
    errors::{Diagnostics, ErrorDiagnosticInfo},
    token::{Token, TokenKind},
    Span,
};

#[derive(Clone, Debug)]
pub enum LexingError<'a> {
    UnknownChar(Span<'a>),
}

impl<'a> Diagnostics<'a> for LexingError<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a> {
        match self {
            Self::UnknownChar(context) => ErrorDiagnosticInfo {
                code: "L001".to_owned(),
                overview: "failed to process unknown character".to_owned(),
                details: "this character is invalid in Go or unsupported".to_owned(),
                context: Some(context.clone()),
            },
        }
    }
}

pub struct Lexer<'a> {
    src: Chars<'a>, // cannot use Peekable<Chars> as it doesn't support .as_str()

    offset: usize, // 0-indexed, from start of src (*not* start of line)
    line: usize,   // 1-indexed
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars(),

            offset: 0,
            line: 1,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        // cloning Chars<'_> is cheap
        self.src.clone().next()
    }

    fn read_char(&mut self) -> Option<char> {
        if let Some(ch) = self.src.next() {
            self.offset += 1;

            if ch == '\n' {
                self.line += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    fn read_span(&mut self) -> Option<Span<'a>> {
        let (original_offset, original_line) = (self.offset, self.line);

        let view = self.src.as_str();

        if self.read_char().is_some() {
            Some(Span::new(&view[..1], original_offset, original_line))
        } else {
            None
        }
    }

    fn read_while<F>(&mut self, cond: F) -> Span<'a>
    where
        F: Fn(char) -> bool,
    {
        let (original_offset, original_line) = (self.offset, self.line);

        let view = self.src.as_str();
        let mut len = 0;
        while self.peek_char().map_or(false, &cond) {
            len += 1;
            self.read_char(); // advance iterator
        }

        Span::new(&view[..len], original_offset, original_line)
    }

    fn identifier_or_keyword(&mut self) -> Token<'a> {
        let ident = self.read_while(|ch| is_letter(ch) || is_unicode_digit(ch));

        Token::from_identifier_or_keyword(ident)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexingError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! single_char_token {
            ($kind:expr) => {
                Token::new($kind, self.read_span().unwrap())
            };
        }

        let token = match self.peek_char() {
            Some(';') => single_char_token!(TokenKind::SemiColon),

            Some(',') => single_char_token!(TokenKind::Comma),
            Some('.') => single_char_token!(TokenKind::Period),
            Some('=') => single_char_token!(TokenKind::Assign),

            Some('(') => single_char_token!(TokenKind::ParenL),
            Some(')') => single_char_token!(TokenKind::ParenR),

            Some(ch) if is_letter(ch) => self.identifier_or_keyword(),
            Some(ch) if is_whitespace(ch) => {
                self.read_char(); // advance iterator
                return self.next();
            }
            Some(_) => return Some(Err(LexingError::UnknownChar(self.read_span().unwrap()))),
            None => return None,
        };

        Some(Ok(token))
    }
}

// character utility functions, as defined by Go spec

fn is_letter(ch: char) -> bool {
    ch.is_letter() || ch == '_'
}

fn is_unicode_digit(ch: char) -> bool {
    ch.is_number_decimal()
}

fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    fn lex(src: &str) -> Result<Vec<Token<'_>>, LexingError<'_>> {
        Lexer::new(src).collect::<Result<Vec<_>, _>>()
    }

    #[test]
    fn package() {
        assert_eq!(
            vec![
                Token {
                    span: Span::new("package", 2, 1),
                    kind: TokenKind::Package,
                },
                Token {
                    span: Span::new("hello", 16, 3),
                    kind: TokenKind::Ident,
                }
            ],
            lex("  package    \t\n\nhello").unwrap(),
        )
    }
}
