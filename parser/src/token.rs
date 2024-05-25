use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    SemiColon, // ;

    Comma,    // ,
    Period,   // .
    Colon,    // :
    Ellipsis, // ...

    ParenL,  // (
    ParenR,  // )
    SquareL, // [
    SquareR, // ]
    CurlyL,  // {
    CurlyR,  // }

    Plus,    // +
    Minus,   // -
    Star,    //_*
    Slash,   // /
    Percent, // %
    Caret,   // ^
    Excl,    //_!

    Amp,        // &
    Pipe,       // |
    DoubleAmp,  // &&
    DoublePipe, // ||
    AmpCaret,   // &^

    DoubleEq, // ==
    NotEq,    //_!=
    Lt,       // <
    Gt,       // >
    LtEq,     // <=
    GtEq,     // >=
    DoubleLt, // <<
    DoubleGt, // >>
    LtMinus,  // <-

    PlusPlus,   // ++
    MinusMinus, // --

    Assign,         // =
    ColonAssign,    // :=
    PlusAssign,     // +=
    MinusAssign,    // -=
    StarAssign,     //_*=
    SlashAssign,    // /=
    PercentAssign,  // %=
    CaretAssign,    // ^=
    AmpAssign,      // &=
    PipeAssign,     // |=
    DoubleLtAssign, // <<=
    DoubleGtAssign, // >>=
    AmpCaretAssign, // &^=

    Int(u64), // 3

    Ident,

    // keywords
    Const,
    Else,
    Func,
    If,
    Package,
    Var,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span<'a>,
    pub annotation: Option<Box<Annotation<'a>>>, // box to prevent bloating size
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, span: Span<'a>) -> Self {
        Self {
            kind,
            span,
            annotation: None,
        }
    }

    pub fn from_identifier_or_keyword(span: Span<'a>, annotation: Option<Annotation<'a>>) -> Self {
        let kind = match span.content {
            "const" => TokenKind::Const,
            "else" => TokenKind::Else,
            "func" => TokenKind::Func,
            "if" => TokenKind::If,
            "package" => TokenKind::Package,
            "var" => TokenKind::Var,
            _ => TokenKind::Ident,
        };

        Self {
            kind,
            span,
            annotation: annotation.map(Box::new),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation<'a> {
    pub scope: &'a str,
    pub labels: Vec<&'a str>,
}
