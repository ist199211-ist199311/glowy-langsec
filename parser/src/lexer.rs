use std::{num::ParseIntError, str::Chars};

use finl_unicode::categories::CharacterCategories;

use crate::{
    errors::{Diagnostics, ErrorDiagnosticInfo},
    token::{Token, TokenKind},
    Span,
};

#[derive(Clone, Debug)]
pub enum LexingError<'a> {
    UnknownChar(Span<'a>),
    InvalidNumberLiteralMode(Span<'a>),
    InvalidNumberLiteralChar(Span<'a>),
    IntParseFailure(Span<'a>, ParseIntError),
}

impl<'a> Diagnostics<'a> for LexingError<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a> {
        macro_rules! s {
            ($lit:expr) => {
                $lit.to_owned()
            };
        }

        match self {
            Self::UnknownChar(context) => ErrorDiagnosticInfo {
                code: s!("L001"),
                overview: s!("failed to process unknown character"),
                details: s!("this character is invalid in Go or unsupported"),
                context: Some(context.clone()),
            },
            Self::InvalidNumberLiteralMode(context) => ErrorDiagnosticInfo {
                code: s!("L002"),
                overview: s!("failed to process unknown number literal mode"),
                details: s!("this kind of literal is not supported (use 'b', 'o', or 'x')"),
                context: Some(context.clone()),
            },
            Self::InvalidNumberLiteralChar(context) => ErrorDiagnosticInfo {
                code: s!("L003"),
                overview: s!("failed to process unknown number literal character"),
                details: s!("this character is not valid for the given literal mode"),
                context: Some(context.clone()),
            },
            Self::IntParseFailure(context, err) => ErrorDiagnosticInfo {
                code: s!("L004"),
                overview: s!("failed to parse integer literal"),
                details: err.to_string(),
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

    fn number_literal(&mut self) -> Result<Token<'a>, LexingError<'a>> {
        enum NumberLexMode {
            Unknown,
            Set,
            Decimal,
            Binary,
            Octal,
            Hex,
        }

        // TODO: support floats

        // FIXME: potentially abstract this into a `read_while_with_state` or
        // `accumulate_while`? (if used elsewhere)

        let mut mode = NumberLexMode::Unknown;
        let mut read = false; // whether a real digit has been read yet

        let (original_offset, original_line) = (self.offset, self.line);

        let view = self.src.as_str();
        let mut len = 0;

        // TODO: allow separating _'s (only between consecutive digits!)
        while let Some(ch) = self.peek_char() {
            match mode {
                NumberLexMode::Unknown if ch == '0' => mode = NumberLexMode::Set,
                NumberLexMode::Decimal | NumberLexMode::Unknown | NumberLexMode::Set
                    if ch.is_ascii_digit() =>
                {
                    mode = NumberLexMode::Decimal;
                    read = true;
                }
                NumberLexMode::Set => {
                    mode = match ch.to_ascii_lowercase() {
                        'b' => NumberLexMode::Binary,
                        'o' => NumberLexMode::Octal,
                        'x' => NumberLexMode::Hex,
                        _ => {
                            let span = self.read_span().unwrap();
                            return Err(LexingError::InvalidNumberLiteralMode(span));
                        }
                    }
                }
                NumberLexMode::Binary if ch == '0' || ch == '1' => read = true,
                NumberLexMode::Octal if ch.is_digit(8) => read = true,
                NumberLexMode::Hex if ch.is_ascii_hexdigit() => read = true,
                _ => {
                    // if had already read something, unknown char might be another token
                    if read {
                        break;
                    } else {
                        // haven't read anything yet, this is officially an error
                        let span = self.read_span().unwrap();
                        return Err(LexingError::InvalidNumberLiteralChar(span));
                    }
                }
            };

            len += 1;
            self.read_char(); // advance
        }

        let view = &view[..len];
        let span = Span::new(view, original_offset, original_line);

        let (radix, start) = match mode {
            NumberLexMode::Unknown => unreachable!("invoker did not peek first! ran out of tokens"),
            NumberLexMode::Set | NumberLexMode::Decimal => (10, view),
            NumberLexMode::Binary => (2, &view[2..]),
            NumberLexMode::Octal => (8, &view[2..]),
            NumberLexMode::Hex => (16, &view[2..]),
        };

        match u64::from_str_radix(start, radix) {
            Ok(int) => Ok(Token::new(TokenKind::Int(int), span)),
            Err(err) => Err(LexingError::IntParseFailure(span, err)),
        }
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

            // TODO: support floats starting with dot (e.g., `.3`)
            // (this is not trivial since it conflicts with TokenKind::Period)
            Some(ch) if ch.is_ascii_digit() => return Some(self.number_literal()),

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

    #[test]
    fn int_lits() {
        assert_eq!(
            vec![
                Token {
                    kind: TokenKind::Int(3),
                    span: Span {
                        content: "3",
                        offset: 2,
                        line: 1
                    }
                },
                Token {
                    kind: TokenKind::Int(50),
                    span: Span {
                        content: "50",
                        offset: 4,
                        line: 1
                    }
                },
                Token {
                    kind: TokenKind::Int(29),
                    span: Span {
                        content: "0b11101",
                        offset: 7,
                        line: 1
                    }
                },
                Token {
                    kind: TokenKind::Int(505),
                    span: Span {
                        content: "0o771",
                        offset: 15,
                        line: 1
                    }
                },
                Token {
                    kind: TokenKind::Int(3909),
                    span: Span {
                        content: "0xf45",
                        offset: 21,
                        line: 1
                    }
                },
                Token {
                    kind: TokenKind::Int(123),
                    span: Span {
                        content: "0123",
                        offset: 28,
                        line: 2
                    }
                },
                Token {
                    kind: TokenKind::Int(0),
                    span: Span {
                        content: "0",
                        offset: 33,
                        line: 2
                    }
                }
            ],
            lex("\t 3 50 0b11101 0o771 0xf45\n 0123 0").unwrap()
        )
    }
}
