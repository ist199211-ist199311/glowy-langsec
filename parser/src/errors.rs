use crate::{
    lexer::LexingError,
    token::{Token, TokenKind},
    Span,
};

#[derive(Clone, Debug)]
pub enum ParsingError<'a> {
    Lexing(LexingError<'a>),
    UnexpectedTokenKind {
        expected: TokenKind,
        found: Option<Token<'a>>,      // None means EOF
        context: Option<&'static str>, // for error message
    },
    UnexpectedConstruct {
        expected: &'static str,
        found: Option<Token<'a>>, // None means EOF
    },
}

impl<'a> From<LexingError<'a>> for ParsingError<'a> {
    fn from(err: LexingError<'a>) -> Self {
        Self::Lexing(err)
    }
}

pub struct ErrorDiagnosticInfo<'a> {
    pub code: String,
    pub overview: String,
    pub details: String,
    pub context: Option<Span<'a>>,
}

pub trait Diagnostics<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a>;
}

impl<'a> Diagnostics<'a> for ParsingError<'a> {
    fn diagnostics(&self) -> ErrorDiagnosticInfo<'a> {
        macro_rules! s {
            ($lit:expr) => {
                $lit.to_owned()
            };
        }

        match self {
            Self::Lexing(e) => e.diagnostics(),
            Self::UnexpectedTokenKind {
                expected,
                found,
                context,
            } => ErrorDiagnosticInfo {
                code: s!("P001"),
                overview: if let Some(ctx) = context {
                    format!("unexpected token in {ctx}")
                } else {
                    s!("unexpected token")
                },
                details: format!(
                    "expected a token of kind {:?}, but found {}",
                    expected,
                    found
                        .as_ref()
                        .map(|t| format!("{:?}", t.kind))
                        .unwrap_or(s!("end-of-file"))
                ),
                context: found.clone().map(|t| t.span),
            },
            Self::UnexpectedConstruct { expected, found } => ErrorDiagnosticInfo {
                code: s!("P002"),
                overview: s!("unexpected construct"),
                details: format!(
                    "expected {}, but found {}",
                    expected,
                    found
                        .as_ref()
                        .map(|t| format!("a token of kind {:?}", t.kind))
                        .unwrap_or(s!("end-of-file"))
                ),
                context: found.clone().map(|t| t.span),
            },
        }
    }
}
