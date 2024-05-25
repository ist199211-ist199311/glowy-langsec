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

type LResult<'a> = Result<Token<'a>, LexingError<'a>>;

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
            self.offset += ch.len_utf8();

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

        if let Some(ch) = self.read_char() {
            let n = ch.len_utf8();
            Some(Span::new(&view[..n], original_offset, original_line))
        } else {
            None
        }
    }

    fn accumulate_while<F, S>(&mut self, initial: S, func: F) -> (Span<'a>, S)
    where
        F: Fn(char, &mut S, &mut Self) -> bool, // FIXME: rewrite using FnMut?
    {
        let (original_offset, original_line) = (self.offset, self.line);

        let view = self.src.as_str();
        let mut len = 0;
        let mut state = initial;
        while let Some(ch) = self.peek_char() {
            if !func(ch, &mut state, self) {
                break;
            }
            len += ch.len_utf8();
            self.read_char(); // advance iterator
        }

        let span = Span::new(&view[..len], original_offset, original_line);

        (span, state)
    }

    fn read_while<F>(&mut self, cond: F) -> Span<'a>
    where
        F: Fn(char) -> bool,
    {
        self.accumulate_while((), |ch, _, _| cond(ch)).0
    }

    fn read_n<const N: usize>(&mut self) -> Span<'a> {
        let (span, _) = self.accumulate_while(0, |_, count, _| {
            if *count < N {
                *count += 1;
                true
            } else {
                false
            }
        });

        span
    }

    fn skip_comments(&mut self) {
        // cloned so we can peek freely
        let mut it = self.src.clone();

        if let Some('/') = it.next() {
            match it.next() {
                Some('/') => {
                    // line comment

                    self.read_n::<2>(); // step over //
                    self.read_while(|ch| ch != '\n');
                }
                Some('*') => {
                    // general comment

                    self.read_n::<2>(); // step over /*
                    loop {
                        self.read_while(|ch| ch != '*');
                        self.read_char(); // step over *
                        if let Some('/') = self.read_char() {
                            break;
                        }
                    }
                }
                _ => {} // not a comment
            }
        }
    }

    fn identifier_or_keyword(&mut self) -> Token<'a> {
        let ident = self.read_while(|ch| is_letter(ch) || is_unicode_digit(ch));

        Token::from_identifier_or_keyword(ident)
    }

    fn number_literal(&mut self) -> LResult<'a> {
        enum NumberLexMode {
            Unknown,
            Set,
            Decimal,
            Binary,
            Octal,
            Hex,
        }

        struct NumberLexState<'a> {
            mode: NumberLexMode,
            read: bool, // whether a real digit has been read yet
            err: Option<LexingError<'a>>,
        }

        // TODO: support floats
        // TODO: allow separating _'s (only between consecutive digits!)

        let (span, state) = self.accumulate_while(
            NumberLexState {
                mode: NumberLexMode::Unknown,
                read: false,
                err: None,
            },
            |ch, state, lexer| {
                match state.mode {
                    NumberLexMode::Unknown if ch == '0' => state.mode = NumberLexMode::Set,
                    NumberLexMode::Decimal | NumberLexMode::Unknown | NumberLexMode::Set
                        if ch.is_ascii_digit() =>
                    {
                        state.mode = NumberLexMode::Decimal;
                        state.read = true;
                    }
                    NumberLexMode::Set => {
                        state.mode = match ch.to_ascii_lowercase() {
                            'b' => NumberLexMode::Binary,
                            'o' => NumberLexMode::Octal,
                            'x' => NumberLexMode::Hex,
                            _ => {
                                let span = lexer.read_span().unwrap();
                                state.err = Some(LexingError::InvalidNumberLiteralMode(span));
                                return false;
                            }
                        }
                    }
                    NumberLexMode::Binary if ch == '0' || ch == '1' => state.read = true,
                    NumberLexMode::Octal if ch.is_digit(8) => state.read = true,
                    NumberLexMode::Hex if ch.is_ascii_hexdigit() => state.read = true,
                    _ => {
                        // if had already read something, unknown char might be another token
                        if state.read {
                            return false;
                        } else {
                            // haven't read anything yet, this is officially an error
                            let span = lexer.read_span().unwrap();
                            state.err = Some(LexingError::InvalidNumberLiteralChar(span));
                            return false;
                        }
                    }
                }

                true
            },
        );

        if let Some(err) = state.err {
            return Err(err);
        };

        let (radix, start) = match state.mode {
            NumberLexMode::Unknown => unreachable!("invoker did not peek first! ran out of tokens"),
            NumberLexMode::Set | NumberLexMode::Decimal => (10, span.content),
            NumberLexMode::Binary => (2, &span.content[2..]),
            NumberLexMode::Octal => (8, &span.content[2..]),
            NumberLexMode::Hex => (16, &span.content[2..]),
        };

        match u64::from_str_radix(start, radix) {
            Ok(int) => Ok(Token::new(TokenKind::Int(int), span)),
            Err(err) => Err(LexingError::IntParseFailure(span, err)),
        }
    }

    fn period_or_ellipsis(&mut self) -> Token<'a> {
        // cannot use greedy since ".." is not a valid token..

        // we can't use &view[..3] == "..." because ..3 might fall
        // outside char boundaries, e.g. "..ü" would panic
        let upcoming: Vec<_> = self.src.clone().take(3).collect();

        if upcoming.len() == 3 && upcoming.iter().all(|x| *x == '.') {
            Token::new(TokenKind::Ellipsis, self.read_n::<3>())
        } else if upcoming.first() == Some(&'.') {
            Token::new(TokenKind::Period, self.read_span().unwrap())
        } else {
            unreachable!("invoker code did not check for a period!")
        }
    }

    fn greedy(&mut self, tree: &TokenOptionsTree<'static>) -> Token<'a> {
        // cannot pass tree directly as initial state since the first
        // iteration needs to take place before any checking so that
        // the first char (already peeked) is included in the final span

        let (span, node) = self.accumulate_while(None, move |ch, state, _| {
            if let Some(&TokenOptionsTree { options, .. }) = state {
                for (key, branch) in options {
                    if ch == *key {
                        *state = Some(branch);
                        return true;
                    }
                }

                false
            } else {
                *state = Some(tree);

                true
            }
        });

        Token::new(node.unwrap().base.clone(), span)
    }
}

struct TokenOptionsTree<'a> {
    base: TokenKind,
    options: &'a [(char, TokenOptionsTree<'a>)],
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! single_char_token {
            ($kind:expr) => {
                Token::new($kind, self.read_span().unwrap())
            };
        }

        macro_rules! tree {
            ($base:expr, $options:expr) => {
                TokenOptionsTree {
                    base: $base,
                    options: $options,
                }
            };
        }

        macro_rules! single_or_eq {
            ($single:expr, $eq:expr) => {
                self.greedy(&tree!($single, &[('=', tree!($eq, &[]))]))
            };
        }

        macro_rules! double_or_eq {
            ($ch:expr, $single:expr, $double:expr, $eq:expr) => {
                self.greedy(&tree!(
                    $single,
                    &[($ch, tree!($double, &[])), ('=', tree!($eq, &[])),]
                ))
            };
        }

        self.skip_comments();

        let token = match self.peek_char() {
            Some(';') => single_char_token!(TokenKind::SemiColon),
            Some(',') => single_char_token!(TokenKind::Comma),
            Some('(') => single_char_token!(TokenKind::ParenL),
            Some(')') => single_char_token!(TokenKind::ParenR),
            Some('[') => single_char_token!(TokenKind::SquareL),
            Some(']') => single_char_token!(TokenKind::SquareR),
            Some('{') => single_char_token!(TokenKind::CurlyL),
            Some('}') => single_char_token!(TokenKind::CurlyR),

            Some(':') => single_or_eq!(TokenKind::Colon, TokenKind::ColonAssign),
            Some('*') => single_or_eq!(TokenKind::Star, TokenKind::StarAssign),
            Some('/') => single_or_eq!(TokenKind::Slash, TokenKind::SlashAssign),
            Some('%') => single_or_eq!(TokenKind::Percent, TokenKind::PercentAssign),
            Some('^') => single_or_eq!(TokenKind::Caret, TokenKind::CaretAssign),
            Some('!') => single_or_eq!(TokenKind::Excl, TokenKind::NotEq),
            Some('=') => single_or_eq!(TokenKind::Assign, TokenKind::DoubleEq),

            Some('+') => double_or_eq!(
                '+',
                TokenKind::Plus,
                TokenKind::PlusPlus,
                TokenKind::PlusAssign
            ),
            Some('-') => double_or_eq!(
                '-',
                TokenKind::Minus,
                TokenKind::MinusMinus,
                TokenKind::MinusAssign
            ),
            Some('|') => double_or_eq!(
                '|',
                TokenKind::Pipe,
                TokenKind::DoublePipe,
                TokenKind::PipeAssign
            ),

            Some('.') => self.period_or_ellipsis(),

            Some('&') => self.greedy(&tree!(
                TokenKind::Amp,
                &[
                    ('&', tree!(TokenKind::DoubleAmp, &[])),
                    ('=', tree!(TokenKind::AmpAssign, &[])),
                    (
                        '^',
                        tree!(
                            TokenKind::AmpCaret,
                            &[('=', tree!(TokenKind::AmpCaretAssign, &[]))]
                        )
                    ),
                ]
            )),
            Some('<') => self.greedy(&tree!(
                TokenKind::Lt,
                &[
                    ('=', tree!(TokenKind::LtEq, &[])),
                    ('-', tree!(TokenKind::LtMinus, &[])),
                    (
                        '<',
                        tree!(
                            TokenKind::DoubleLt,
                            &[('=', tree!(TokenKind::DoubleLtAssign, &[]))]
                        )
                    )
                ]
            )),
            Some('>') => self.greedy(&tree!(
                TokenKind::Gt,
                &[
                    ('=', tree!(TokenKind::GtEq, &[])),
                    (
                        '>',
                        tree!(
                            TokenKind::DoubleGt,
                            &[('=', tree!(TokenKind::DoubleGtAssign, &[]))]
                        )
                    )
                ]
            )),

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
                    span: Span::new("3", 2, 1)
                },
                Token {
                    kind: TokenKind::Int(50),
                    span: Span::new("50", 4, 1)
                },
                Token {
                    kind: TokenKind::Int(29),
                    span: Span::new("0b11101", 7, 1)
                },
                Token {
                    kind: TokenKind::Int(505),
                    span: Span::new("0o771", 15, 1)
                },
                Token {
                    kind: TokenKind::Int(3909),
                    span: Span::new("0xf45", 21, 1)
                },
                Token {
                    kind: TokenKind::Int(123),
                    span: Span::new("0123", 28, 2)
                },
                Token {
                    kind: TokenKind::Int(0),
                    span: Span::new("0", 33, 2)
                }
            ],
            lex("\t 3 50 0b11101 0o771 0xf45\n 0123 0").unwrap()
        )
    }

    #[test]
    fn greedy() {
        assert_eq!(
            vec![
                Token {
                    kind: TokenKind::Gt,
                    span: Span::new(">", 0, 1)
                },
                Token {
                    kind: TokenKind::Excl,
                    span: Span::new("!", 2, 1)
                },
                Token {
                    kind: TokenKind::DoubleEq,
                    span: Span::new("==", 4, 1)
                },
                Token {
                    kind: TokenKind::NotEq,
                    span: Span::new("!=", 7, 1)
                },
                Token {
                    kind: TokenKind::AmpCaret,
                    span: Span::new("&^", 10, 1)
                },
                Token {
                    kind: TokenKind::AmpCaretAssign,
                    span: Span::new("&^=", 13, 1)
                },
                Token {
                    kind: TokenKind::Comma,
                    span: Span::new(",", 17, 1)
                },
                Token {
                    kind: TokenKind::DoubleGt,
                    span: Span::new(">>", 19, 1)
                },
                Token {
                    kind: TokenKind::Gt,
                    span: Span::new(">", 21, 1)
                }
            ],
            lex("> ! == != &^ &^= , >>>").unwrap()
        )
    }
}
