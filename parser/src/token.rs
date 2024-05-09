use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    SemiColon(bool), // true if real (i.e., not auto-inserted)

    Ident,

    // keywords
    Package,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span<'a>,
}

impl<'a> Token<'a> {
    pub fn from_identifier_or_keyword(span: Span<'a>) -> Self {
        let kind = match span.content {
            "package" => TokenKind::Package,
            _ => TokenKind::Ident,
        };

        Self { kind, span }
    }
}
