use std::{collections::VecDeque, num::ParseIntError, str::Chars};

use finl_unicode::categories::CharacterCategories;
use regex::Regex;

use crate::{
    errors::{Diagnostics, ErrorDiagnosticInfo},
    token::{Annotation, Token, TokenKind},
    Span,
};

#[derive(Clone, Debug, PartialEq)]
pub enum LexingError<'a> {
    UnknownChar(Span<'a>),
    InvalidNumberLiteralChar(Span<'a>),
    IntParseFailure(Span<'a>, ParseIntError),
    MultipleCharactersInRune(Span<'a>),
    EmptyRune(Span<'a>),
    LineBreakInString(Span<'a>),
    InvalidStringEscapeSequence(Span<'a>),
    UnclosedString,
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
            Self::InvalidNumberLiteralChar(context) => ErrorDiagnosticInfo {
                code: s!("L002"),
                overview: s!("failed to process unknown number literal character"),
                details: s!("this character is not valid for the given literal mode"),
                context: Some(context.clone()),
            },
            Self::IntParseFailure(context, err) => ErrorDiagnosticInfo {
                code: s!("L003"),
                overview: s!("failed to parse integer literal"),
                details: err.to_string(),
                context: Some(context.clone()),
            },
            Self::MultipleCharactersInRune(context) => ErrorDiagnosticInfo {
                code: s!("L004"),
                overview: s!("multiple characters in rune"),
                details: s!("found more than one character in the given rune"),
                context: Some(context.clone()),
            },
            Self::EmptyRune(context) => ErrorDiagnosticInfo {
                code: s!("L005"),
                overview: s!("empty rune"),
                details: s!("found no characters in the given rune"),
                context: Some(context.clone()),
            },
            Self::LineBreakInString(context) => ErrorDiagnosticInfo {
                code: s!("L006"),
                overview: s!("line break in string"),
                details: s!("the newline character (\\n) is not allowed in string literals"),
                context: Some(context.clone()),
            },
            Self::InvalidStringEscapeSequence(context) => ErrorDiagnosticInfo {
                code: s!("L007"),
                overview: s!("invalid escape sequence"),
                details: s!("escape sequence in string is invalid"),
                context: Some(context.clone()),
            },
            Self::UnclosedString => ErrorDiagnosticInfo {
                code: s!("L008"),
                overview: s!("unclosed string"),
                details: s!("reached EOF before finding a closing string delimiter"),
                context: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    src: Chars<'a>, // cannot use Peekable<Chars> as it doesn't support .as_str()

    offset: usize, // 0-indexed, from start of src (*not* start of line)
    line: usize,   // 1-indexed

    last_token_kind: Option<TokenKind>,
    queue: VecDeque<Token<'a>>,

    annotation_regex: Regex, // prevent constant recompilation (slow)
    last_annotation: Option<Annotation<'a>>, // prevent clearing by whitespace

    enable_implicit_semicolon: bool, // whether to enable implicit semicolon insertion
}

type LResult<'a> = Result<Token<'a>, LexingError<'a>>;

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.chars(),

            offset: 0,
            line: 1,

            last_token_kind: None,
            queue: VecDeque::new(),

            annotation_regex: Regex::new(r#"glowy::(?P<scope>\w+)::\{(?P<tags>[^}]*)\}"#).unwrap(),
            last_annotation: None,

            enable_implicit_semicolon: true,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        // cloning Chars<'_> is cheap
        self.src.clone().next()
    }

    fn read_char(&mut self) -> Option<char> {
        let view = self.src.as_str();
        let (original_offset, original_line) = (self.offset, self.line);

        if let Some(ch) = self.src.next() {
            self.offset += ch.len_utf8();

            if ch == '\n' {
                self.line += 1;

                if self.enable_implicit_semicolon
                    && self
                        .last_token_kind
                        .as_ref()
                        .map(TokenKind::allows_implicit_semicolon)
                        .unwrap_or(false)
                {
                    // newline is guaranteed single-byte, no panic
                    let span = Span::new(&view[..1], original_offset, original_line);
                    self.queue.push_back(Token::new(TokenKind::SemiColon, span));
                }
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
                    let text = self.read_while(|ch| ch != '\n').content;

                    if let Some(captures) = self.annotation_regex.captures(text) {
                        let scope = &text[captures.name("scope").unwrap().range()];
                        let tags = text[captures.name("tags").unwrap().range()]
                            .split(',')
                            .map(str::trim)
                            .filter(|tag| !tag.is_empty())
                            .collect();

                        self.last_annotation = Some(Annotation { scope, tags });
                    }
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

        Token::from_identifier_or_keyword(ident, &mut self.last_annotation)
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
                            _ => return false, // this is probably another token
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

    fn string_like_literal(&mut self) -> LResult<'a> {
        enum StringLexMode {
            Unknown,
            Rune,              // unicode character ('a')
            InterpretedString, // "hello\nworld"
            RawString,         // `hello world`
        }

        enum StringLexEscapeMode {
            Normal,
            Backslash,
            EscapedUnicode {
                value: u32,
                read_count: u8,
                radix: u32,
                expected_count: u8,
                max_value: u32,
            },
        }

        struct StringLexState<'a> {
            mode: StringLexMode,
            escape_mode: StringLexEscapeMode,
            last_char: Option<char>,
            string: String,
            finished: bool, // whether closing delimiter has been read
            err: Option<LexingError<'a>>,
        }

        macro_rules! push_char {
            ($state:expr, $char:expr) => {{
                if let Some(c) = $state.last_char {
                    $state.string.push(c);
                }
                $state.last_char = Some($char);
                $state.escape_mode = StringLexEscapeMode::Normal;
            }};
        }

        let prev_implicit_semicolon = self.enable_implicit_semicolon;
        self.enable_implicit_semicolon = false;

        let (span, state) = self.accumulate_while(
            StringLexState {
                mode: StringLexMode::Unknown,
                escape_mode: StringLexEscapeMode::Normal,
                last_char: None,
                string: String::new(),
                finished: false,
                err: None,
            },
            |ch, state, lexer| {
                if state.finished {
                    return false;
                }
                match &mut state.escape_mode {
                    StringLexEscapeMode::Normal => match (ch, &state.mode) {
                        ('\'', StringLexMode::Unknown) => state.mode = StringLexMode::Rune,
                        ('"', StringLexMode::Unknown) => {
                            state.mode = StringLexMode::InterpretedString
                        }
                        ('`', StringLexMode::Unknown) => state.mode = StringLexMode::RawString,
                        (_, StringLexMode::Unknown) => unreachable!(
                            "function string_like_literal called on non-string boundary"
                        ),

                        ('\'', StringLexMode::Rune) => {
                            // end rune
                            state.finished = true;
                        }
                        (_, StringLexMode::Rune) if state.last_char.is_some() => {
                            // rune already has character, but closing quote not found
                            let span = lexer.read_span().unwrap();
                            state.err = Some(LexingError::MultipleCharactersInRune(span));
                            return false;
                        }
                        ('`', StringLexMode::RawString)
                        | ('"', StringLexMode::InterpretedString) => {
                            // end raw and interpreted string
                            if let Some(c) = state.last_char {
                                state.string.push(c);
                                state.last_char = None;
                            }
                            state.finished = true;
                        }
                        ('\\', StringLexMode::Rune) | ('\\', StringLexMode::InterpretedString) => {
                            state.escape_mode = StringLexEscapeMode::Backslash;
                        }
                        ('\n', StringLexMode::Rune) | ('\n', StringLexMode::InterpretedString) => {
                            let span = lexer.read_span().unwrap();
                            state.err = Some(LexingError::LineBreakInString(span));
                            return false;
                        }
                        ('\r', StringLexMode::RawString) => {} // carriage returns are discarded
                        // in raw strings
                        _ => push_char!(state, ch),
                    },
                    StringLexEscapeMode::Backslash => {
                        match (ch, &state.mode) {
                            ('a', _) => push_char!(state, '\u{0007}'),
                            ('b', _) => push_char!(state, '\u{0008}'),
                            ('f', _) => push_char!(state, '\u{000c}'),
                            ('n', _) => push_char!(state, '\n'),
                            ('r', _) => push_char!(state, '\r'),
                            ('t', _) => push_char!(state, '\u{0009}'),
                            ('v', _) => push_char!(state, '\u{000b}'),
                            ('\\', _) => push_char!(state, '\\'),
                            ('\'', StringLexMode::Rune) => push_char!(state, '\''),
                            ('"', StringLexMode::InterpretedString) => push_char!(state, '"'),

                            ('0'..='7', _) => {
                                state.escape_mode = StringLexEscapeMode::EscapedUnicode {
                                    value: ch.to_digit(8).expect("char to be a valid octal digit"),
                                    read_count: 1,
                                    radix: 8,
                                    expected_count: 3,
                                    max_value: u8::MAX as u32,
                                }
                            }
                            ('x', _) => {
                                state.escape_mode = StringLexEscapeMode::EscapedUnicode {
                                    value: 0,
                                    read_count: 0,
                                    radix: 16,
                                    expected_count: 2,
                                    max_value: u8::MAX as u32,
                                }
                            }
                            ('u', _) => {
                                state.escape_mode = StringLexEscapeMode::EscapedUnicode {
                                    value: 0,
                                    read_count: 0,
                                    radix: 16,
                                    expected_count: 4,
                                    max_value: u32::MAX,
                                }
                            }
                            ('U', _) => {
                                state.escape_mode = StringLexEscapeMode::EscapedUnicode {
                                    value: 0,
                                    read_count: 0,
                                    radix: 16,
                                    expected_count: 8,
                                    max_value: u32::MAX,
                                }
                            }

                            (_, _) => {
                                // error: invalid char after backslash
                                let span = lexer.read_span().unwrap();
                                state.err = Some(LexingError::InvalidStringEscapeSequence(span));
                                return false;
                            }
                        }
                    }
                    StringLexEscapeMode::EscapedUnicode {
                        value,
                        read_count,
                        radix,
                        expected_count,
                        max_value,
                    } => {
                        match ch
                            .to_digit(*radix)
                            .and_then(|digit| {
                                value.checked_mul(*radix).and_then(|v| v.checked_add(digit))
                            })
                            .filter(|v| v <= max_value)
                            .and_then(|v| char::from_u32(v).map(|c| (v, c)))
                        {
                            Some((new_value, c)) => {
                                *value = new_value;
                                *read_count += 1;
                                if read_count >= expected_count {
                                    push_char!(state, c);
                                }
                            }
                            None => {
                                // error: invalid digit
                                let span = lexer.read_span().unwrap();
                                state.err = Some(LexingError::InvalidStringEscapeSequence(span));
                                return false;
                            }
                        }
                    }
                }

                true
            },
        );

        self.enable_implicit_semicolon = prev_implicit_semicolon;

        if let Some(err) = state.err {
            return Err(err);
        };

        if !state.finished {
            // reached EOF before closing delimiter
            return Err(LexingError::UnclosedString);
        }

        match &state.mode {
            StringLexMode::Rune => match state.last_char {
                Some(c) => Ok(Token::new(TokenKind::Rune(c), span)),
                None => Err(LexingError::EmptyRune(span)),
            },
            StringLexMode::InterpretedString | StringLexMode::RawString => {
                Ok(Token::new(TokenKind::String(state.string), span))
            }
            StringLexMode::Unknown => {
                unreachable!("function string_like_literal called on non-string boundary")
            }
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

        if let Some(queued) = self.queue.pop_front() {
            self.last_token_kind = Some(queued.kind.clone());
            return Some(Ok(queued));
        }

        self.skip_comments();

        let mut token = match self.peek_char() {
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
            Some(ch) if ch.is_ascii_digit() => match self.number_literal() {
                Ok(token) => token,
                err @ Err(_) => return Some(err),
            },
            Some('\'') | Some('"') | Some('`') => match self.string_like_literal() {
                Ok(token) => token,
                err @ Err(_) => return Some(err),
            },

            Some(ch) if is_letter(ch) => self.identifier_or_keyword(),
            Some(ch) if is_whitespace(ch) => {
                self.read_char(); // advance iterator
                return self.next();
            }
            Some(_) => return Some(Err(LexingError::UnknownChar(self.read_span().unwrap()))),
            None => return None,
        };

        self.last_token_kind = Some(token.kind.clone());

        // FIXME: find a more sane way to pass annotation to function call
        // and short var decl without using these punctuation tokens; if
        // that happens, we can clear last annotation after every token
        // (right before returning). identifier_or_keyword could also go
        // back to calling take directly instead of passing a mutable reference
        if let TokenKind::ColonAssign | TokenKind::ParenL = token.kind {
            token.annotation = self.last_annotation.take().map(Box::new);
        } else if token.kind == TokenKind::SemiColon {
            self.last_annotation.take(); // clear
        }

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
                Token::new(TokenKind::Package, Span::new("package", 2, 1)),
                Token::new(TokenKind::Ident, Span::new("hello", 16, 3))
            ],
            lex("  package    \t\n\nhello").unwrap(),
        )
    }

    #[test]
    fn int_lits() {
        assert_eq!(
            vec![
                Token::new(TokenKind::Int(3), Span::new("3", 2, 1)),
                Token::new(TokenKind::Int(50), Span::new("50", 4, 1)),
                Token::new(TokenKind::Int(29), Span::new("0b11101", 7, 1)),
                Token::new(TokenKind::Int(505), Span::new("0o771", 15, 1)),
                Token::new(TokenKind::Int(3909), Span::new("0xf45", 21, 1)),
                Token::new(TokenKind::SemiColon, Span::new("\n", 26, 1)),
                Token::new(TokenKind::Int(123), Span::new("0123", 28, 2)),
                Token::new(TokenKind::Int(0), Span::new("0", 33, 2))
            ],
            lex("\t 3 50 0b11101 0o771 0xf45\n 0123 0").unwrap()
        )
    }

    #[test]
    fn rune_lits() {
        assert_eq!(
            vec![
                Token::new(TokenKind::Rune('a'), Span::new("'a'", 2, 1)),
                Token::new(TokenKind::Rune('\u{0007}'), Span::new("'\\a'", 6, 1)),
                Token::new(TokenKind::Rune('\n'), Span::new("'\\n'", 11, 1)),
                Token::new(TokenKind::SemiColon, Span::new("\n", 15, 1)),
                Token::new(TokenKind::Rune('\''), Span::new("'\\''", 17, 2)),
                Token::new(TokenKind::Rune('ä'), Span::new("'ä'", 22, 2)),
                Token::new(TokenKind::Rune('本'), Span::new("'本'", 27, 2)),
                Token::new(TokenKind::Rune('\t'), Span::new("'\\t'", 33, 2)),
                Token::new(TokenKind::Rune('\t'), Span::new("'\t'", 38, 2)),
                Token::new(TokenKind::Rune('\0'), Span::new("'\\000'", 42, 2)),
                Token::new(TokenKind::Rune('\x07'), Span::new("'\\007'", 49, 2)),
                Token::new(TokenKind::Rune('\u{ff}'), Span::new("'\\377'", 56, 2)),
                Token::new(TokenKind::Rune('\u{07}'), Span::new("'\\x07'", 63, 2)),
                Token::new(TokenKind::Rune('\u{ff}'), Span::new("'\\xff'", 70, 2)),
                Token::new(TokenKind::Rune('\u{12e4}'), Span::new("'\\u12e4'", 77, 2)),
                Token::new(
                    TokenKind::Rune('\u{101234}'),
                    Span::new("'\\U00101234'", 86, 2)
                ),
            ],
            lex(
                "\t 'a' '\\a' '\\n'\n '\\'' 'ä' '本' '\\t' '\t' '\\000' '\\007' '\\377' '\\x07' \
                 '\\xff' '\\u12e4' '\\U00101234'  "
            )
            .unwrap()
        );

        assert_eq!(
            Err(LexingError::MultipleCharactersInRune(Span::new("a", 2, 1))),
            lex("'aa'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "k", 2, 1
            ))),
            lex("'\\k'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "'", 4, 1
            ))),
            lex("'\\xa'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "'", 3, 1
            ))),
            lex("'\\0'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "0", 4, 1
            ))),
            lex("'\\400'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "F", 6, 1
            ))),
            lex("'\\uDFFF'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "0", 10, 1
            ))),
            lex("'\\U00110000'")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "\"", 2, 1
            ))),
            lex("'\\\"'")
        );
        assert_eq!(
            Err(LexingError::EmptyRune(Span::new("''", 0, 1))),
            lex("''")
        );
        assert_eq!(Err(LexingError::UnclosedString), lex("'"));
    }

    #[test]
    fn string_lits() {
        macro_rules! s {
            ($lit:expr) => {
                $lit.to_owned()
            };
        }

        assert_eq!(
            vec![
                Token::new(TokenKind::String(s!("abc")), Span::new("`abc`", 4, 1)),
                Token::new(
                    TokenKind::String(s!("\\n\n\\n")),
                    Span::new("`\\n\n\\n`", 10, 1)
                ),
                Token::new(TokenKind::String(s!("\n")), Span::new("\"\\n\"", 18, 2)),
                Token::new(TokenKind::String(s!("\"")), Span::new("\"\\\"\"", 23, 2)),
                Token::new(TokenKind::SemiColon, Span::new("\n", 27, 2)),
                Token::new(
                    TokenKind::String(s!("Hello, world!\n")),
                    Span::new("\"Hello, world!\\n\"", 29, 3)
                ),
                Token::new(
                    TokenKind::String(s!("日本語")),
                    Span::new("\"日本語\"", 47, 3)
                ),
                Token::new(
                    TokenKind::String(s!("\u{65e5}本\u{008a9e}")),
                    Span::new("\"\\u65e5本\\U00008a9e\"", 59, 3)
                ),
                Token::new(
                    TokenKind::String(s!("\u{ff}\u{00FF}")),
                    Span::new("\"\\xff\\u00FF\"", 81, 3)
                ),
                Token::new(TokenKind::String(s!("a\nb")), Span::new("`a\n\rb`", 94, 3)),
                Token::new(TokenKind::String(s!("")), Span::new("\"\"", 101, 4)),
                Token::new(TokenKind::String(s!("")), Span::new("``", 104, 4)),
            ],
            lex(
                "  \t `abc` `\\n\n\\n` \"\\n\" \"\\\"\"\n \"Hello, world!\\n\" \"日本語\" \
                 \"\\u65e5本\\U00008a9e\" \"\\xff\\u00FF\" `a\n\rb` \"\" ``  "
            )
            .unwrap()
        );

        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "0", 6, 1
            ))),
            lex("\"\\uD800\"")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "0", 10, 1
            ))),
            lex("\"\\U00110000\"")
        );
        assert_eq!(
            Err(LexingError::LineBreakInString(Span::new("\n", 2, 1))),
            lex("\"a\nb\"")
        );
        assert_eq!(
            Err(LexingError::LineBreakInString(Span::new("\n", 2, 1))),
            lex("\"a\nb\"")
        );
        assert_eq!(
            Err(LexingError::InvalidStringEscapeSequence(Span::new(
                "'", 2, 1
            ))),
            lex("\"\\'\"")
        );
        assert_eq!(Err(LexingError::UnclosedString), lex("\"aa"));
    }

    #[test]
    fn greedy() {
        assert_eq!(
            vec![
                Token::new(TokenKind::Gt, Span::new(">", 0, 1)),
                Token::new(TokenKind::Excl, Span::new("!", 2, 1)),
                Token::new(TokenKind::DoubleEq, Span::new("==", 4, 1)),
                Token::new(TokenKind::NotEq, Span::new("!=", 7, 1)),
                Token::new(TokenKind::AmpCaret, Span::new("&^", 10, 1)),
                Token::new(TokenKind::AmpCaretAssign, Span::new("&^=", 13, 1)),
                Token::new(TokenKind::Comma, Span::new(",", 17, 1)),
                Token::new(TokenKind::DoubleGt, Span::new(">>", 19, 1)),
                Token::new(TokenKind::Gt, Span::new(">", 21, 1))
            ],
            lex("> ! == != &^ &^= , >>>").unwrap()
        )
    }
}
