use crate::{
    lexer::LexingError,
    token::{Token, TokenKind},
    Span,
};

#[derive(Debug)]
pub enum ParsingError<'a> {
    Lexing(LexingError<'a>),
    UnexpectedToken {
        expected: TokenKind,
        found: Option<Token<'a>>,      // None means EOF
        context: Option<&'static str>, // for error message
    },
    ExtraneousToken(Token<'a>),
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
            Self::UnexpectedToken {
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
            Self::ExtraneousToken(found) => ErrorDiagnosticInfo {
                code: s!("P003"),
                overview: s!("extraneous token"),
                details: format!(
                    "expected end-of-file, but found a token of kind {:?}",
                    found
                ),
                context: Some(found.span.clone()),
            },
        }
    }
}
