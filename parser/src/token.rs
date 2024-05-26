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
    Return,
    Var,
}

impl TokenKind {
    pub fn allows_implicit_semicolon(&self) -> bool {
        matches!(
            self,
            TokenKind::Ident
                | TokenKind::Int(_)
                | TokenKind::Return
                | TokenKind::PlusPlus
                | TokenKind::MinusMinus
                | TokenKind::ParenR
                | TokenKind::SquareR
                | TokenKind::CurlyR
        )
    }
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

    pub fn from_identifier_or_keyword(
        span: Span<'a>,
        annotation: &mut Option<Annotation<'a>>,
    ) -> Self {
        let kind = match span.content {
            "const" => TokenKind::Const,
            "else" => TokenKind::Else,
            "func" => TokenKind::Func,
            "if" => TokenKind::If,
            "package" => TokenKind::Package,
            "return" => TokenKind::Return,
            "var" => TokenKind::Var,
            _ => TokenKind::Ident,
        };

        let annotation = if kind != TokenKind::Ident {
            annotation.take().map(Box::new)
        } else {
            None
        };

        Self {
            kind,
            span,
            annotation,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation<'a> {
    pub scope: &'a str,
    pub labels: Vec<&'a str>,
}
