use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    SemiColon, // ;

    Comma,  // ,
    Period, // .
    Assign, // =

    ParenL, // (
    ParenR, // )

    Int(u64), // 3

    Ident,

    // keywords
    Const,
    Package,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span<'a>,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, span: Span<'a>) -> Self {
        Self { kind, span }
    }

    pub fn from_identifier_or_keyword(span: Span<'a>) -> Self {
        let kind = match span.content {
            "const" => TokenKind::Const,
            "package" => TokenKind::Package,
            _ => TokenKind::Ident,
        };

        Self { kind, span }
    }
}
