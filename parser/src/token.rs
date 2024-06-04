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

    Int(u64),       // 3
    Rune(char),     // 'a'
    String(String), // "hello world"

    Ident,

    // keywords
    Chan,
    Const,
    Else,
    Func,
    Go,
    If,
    Import,
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
                | TokenKind::Rune(_)
                | TokenKind::String(_)
                | TokenKind::Return
                | TokenKind::PlusPlus
                | TokenKind::MinusMinus
                | TokenKind::ParenR
                | TokenKind::SquareR
                | TokenKind::CurlyR
        )
    }

    pub fn admits_annotation(&self) -> bool {
        matches!(
            self,
            // punctuation
            TokenKind::ColonAssign // short var decl
            | TokenKind::ParenL // function call
            | TokenKind::LtMinus // receive

            // keywords
            | TokenKind::Const // const decl
            | TokenKind::Var // var decl
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

    pub fn from_identifier_or_keyword(span: Span<'a>) -> Self {
        let kind = match span.content {
            "chan" => TokenKind::Chan,
            "const" => TokenKind::Const,
            "else" => TokenKind::Else,
            "func" => TokenKind::Func,
            "go" => TokenKind::Go,
            "if" => TokenKind::If,
            "import" => TokenKind::Import,
            "package" => TokenKind::Package,
            "return" => TokenKind::Return,
            "var" => TokenKind::Var,
            _ => TokenKind::Ident,
        };

        Self::new(kind, span)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation<'a> {
    pub scope: &'a str,
    pub tags: Vec<&'a str>,
}
