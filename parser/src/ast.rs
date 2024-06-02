use crate::{Annotation, Span};

#[derive(Debug, PartialEq)]
pub struct SourceFileNode<'a> {
    pub package_clause: PackageClauseNode<'a>,
    pub imports: Vec<ImportNode<'a>>,
    pub top_level_decls: Vec<DeclNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PackageClauseNode<'a> {
    pub id: Span<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ImportNode<'a> {
    pub specs: Vec<ImportSpecNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct ImportSpecNode<'a> {
    pub identifier: Option<Span<'a>>,
    pub path: String,
}

#[derive(Debug, PartialEq)]
pub enum DeclNode<'a> {
    Const {
        specs: Vec<BindingDeclSpecNode<'a>>,
        annotation: Option<Box<Annotation<'a>>>,
    },
    Var {
        specs: Vec<BindingDeclSpecNode<'a>>,
        annotation: Option<Box<Annotation<'a>>>,
    },
    Function(FunctionDeclNode<'a>),
}

impl<'a> From<FunctionDeclNode<'a>> for DeclNode<'a> {
    fn from(node: FunctionDeclNode<'a>) -> Self {
        Self::Function(node)
    }
}

// binding = const or var, since specs look the same for both
#[derive(Debug, PartialEq)]
pub struct BindingDeclSpecNode<'a> {
    pub mapping: Vec<(Span<'a>, ExprNode<'a>)>,
    pub r#type: Option<TypeNode<'a>>,
}

#[derive(Debug)]
pub struct MismatchingBindingDeclSpecListsLength;

impl<'a> BindingDeclSpecNode<'a> {
    pub fn try_new(
        ids: Vec<Span<'a>>,
        exprs: Vec<ExprNode<'a>>,
        r#type: Option<TypeNode<'a>>,
    ) -> Result<Self, MismatchingBindingDeclSpecListsLength> {
        if ids.len() != exprs.len() {
            Err(MismatchingBindingDeclSpecListsLength {})
        } else {
            Ok(Self {
                mapping: ids.into_iter().zip(exprs).collect(),
                r#type,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeNode<'a> {
    Name {
        package: Option<Span<'a>>, // for qualified type names
        id: Span<'a>,
        args: Vec<TypeNode<'a>>,
    },
    Channel {
        r#type: Box<TypeNode<'a>>, // what values can be sent/received
        direction: Option<ChannelDirection>,
    },
    // TODO: Literal
}

#[derive(Debug, PartialEq)]
pub enum ChannelDirection {
    Send,
    Receive,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDeclNode<'a> {
    pub name: Span<'a>,
    // TODO: pub type_params: Vec<___>,
    pub signature: FunctionSignatureNode<'a>,
    /// note: this parser intentionally does not support omitted bodies!
    /// (it would defeat the purpose of information flow control, and
    ///  make parsing much more complicated due to 2 optional elements
    ///  in a row, namely signature result and body)
    pub body: BlockNode<'a>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionSignatureNode<'a> {
    pub params: Vec<FunctionParamDeclNode<'a>>,
    pub result: Option<FunctionResultNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum FunctionResultNode<'a> {
    Single(TypeNode<'a>),
    Params(Vec<FunctionParamDeclNode<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct FunctionParamDeclNode<'a> {
    pub ids: Vec<Span<'a>>,
    pub variadic: bool, // whether type is ...T
    pub r#type: TypeNode<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ExprNode<'a> {
    Name(OperandNameNode<'a>),
    Literal(LiteralNode),
    Call(CallNode<'a>),
    Indexing(IndexingNode<'a>),
    // TODO: more primary expressions...
    UnaryOp {
        kind: UnaryOpKind,
        operand: Box<ExprNode<'a>>,
    },
    BinaryOp {
        kind: BinaryOpKind,
        left: Box<ExprNode<'a>>,
        right: Box<ExprNode<'a>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum UnaryOpKind {
    Identity,   // +x is 0 + x
    Negation,   // -x is 0 - x
    Complement, // ^x is m ^ x for [m = 0b111..11 if x unsigned] or [m = -1 if x signed]
    Not,        //_!x
    Deref,      //_*x
    Address,    // &x
    Receive,    // <-x
}

#[derive(Debug, PartialEq)]
pub enum BinaryOpKind {
    Eq,         // x == y
    NotEq,      // x != y
    Less,       // x < y
    LessEq,     // x <= y
    Greater,    // x > y
    GreaterEq,  // x >= y
    Sum,        // x + y
    Diff,       // x - y
    Product,    // x * y
    Quotient,   // x / y
    Remainder,  // x % y
    ShiftLeft,  // x << y
    ShiftRight, // x >> y
    BitwiseOr,  // x | y
    BitwiseAnd, // x & y
    BitwiseXor, // x ^ y
    BitClear,   // x &^ y (AND NOT)
    LogicalAnd, // x && y
    LogicalOr,  // x || y
}

impl<'a> From<OperandNameNode<'a>> for ExprNode<'a> {
    fn from(node: OperandNameNode<'a>) -> Self {
        Self::Name(node)
    }
}

impl From<LiteralNode> for ExprNode<'_> {
    fn from(node: LiteralNode) -> Self {
        Self::Literal(node)
    }
}

impl<'a> From<CallNode<'a>> for ExprNode<'a> {
    fn from(node: CallNode<'a>) -> Self {
        Self::Call(node)
    }
}

impl<'a> From<IndexingNode<'a>> for ExprNode<'a> {
    fn from(node: IndexingNode<'a>) -> Self {
        Self::Indexing(node)
    }
}

#[derive(Debug, PartialEq)]
pub struct OperandNameNode<'a> {
    pub package: Option<Span<'a>>, // for qualified operand names
    pub id: Span<'a>,
}

#[derive(Debug, PartialEq)]
pub enum LiteralNode {
    Int(u64),
    Rune(char),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct CallNode<'a> {
    pub func: Box<ExprNode<'a>>,
    pub args: Vec<ExprNode<'a>>,
    pub variadic: bool, // whether the last argument is "x..."
    pub annotation: Option<Box<Annotation<'a>>>,
}

#[derive(Debug, PartialEq)]
pub struct IndexingNode<'a> {
    pub expr: Box<ExprNode<'a>>,
    pub index: Box<ExprNode<'a>>,
}

pub type BlockNode<'a> = Vec<StatementNode<'a>>;

#[derive(Debug, PartialEq)]
pub enum StatementNode<'a> {
    // simple
    Empty,
    Expr(ExprNode<'a>),
    Send(SendNode<'a>),
    Inc(ExprNode<'a>),
    Dec(ExprNode<'a>),
    Assignment(AssignmentNode<'a>),
    ShortVarDecl(ShortVarDeclNode<'a>),

    // non-simple
    Decl(DeclNode<'a>),
    If(IfNode<'a>),
    Block(BlockNode<'a>),
    Return(Vec<ExprNode<'a>>),
    Go(ExprNode<'a>),
}

impl<'a> From<ExprNode<'a>> for StatementNode<'a> {
    fn from(node: ExprNode<'a>) -> Self {
        Self::Expr(node)
    }
}

impl<'a> From<SendNode<'a>> for StatementNode<'a> {
    fn from(node: SendNode<'a>) -> Self {
        Self::Send(node)
    }
}

impl<'a> From<AssignmentNode<'a>> for StatementNode<'a> {
    fn from(node: AssignmentNode<'a>) -> Self {
        Self::Assignment(node)
    }
}

impl<'a> From<ShortVarDeclNode<'a>> for StatementNode<'a> {
    fn from(node: ShortVarDeclNode<'a>) -> Self {
        Self::ShortVarDecl(node)
    }
}

impl<'a> From<DeclNode<'a>> for StatementNode<'a> {
    fn from(node: DeclNode<'a>) -> Self {
        Self::Decl(node)
    }
}

impl<'a> From<IfNode<'a>> for StatementNode<'a> {
    fn from(node: IfNode<'a>) -> Self {
        Self::If(node)
    }
}

impl<'a> From<BlockNode<'a>> for StatementNode<'a> {
    fn from(node: BlockNode<'a>) -> Self {
        Self::Block(node)
    }
}

#[derive(Debug, PartialEq)]
pub struct SendNode<'a> {
    pub channel: ExprNode<'a>,
    pub expr: ExprNode<'a>,
}

#[derive(Debug, PartialEq)]
pub struct AssignmentNode<'a> {
    pub kind: AssignmentKind,
    pub lhs: Vec<ExprNode<'a>>,
    pub rhs: Vec<ExprNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum AssignmentKind {
    Simple,     //   =
    Sum,        //  +=
    Diff,       //  -=
    Product,    //_ *=
    Quotient,   //  /=
    Remainder,  //  %=
    ShiftLeft,  // <<=
    ShiftRight, // >>=
    BitwiseOr,  //  |=
    BitwiseAnd, //  &=
    BitwiseXor, //  ^=
    BitClear,   // &^=
}

#[derive(Debug, PartialEq)]
pub struct ShortVarDeclNode<'a> {
    pub ids: Vec<Span<'a>>,
    pub exprs: Vec<ExprNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct IfNode<'a> {
    // TODO: pub stmt: Box<StatementNode<'a>>,
    pub cond: ExprNode<'a>,
    pub then: BlockNode<'a>,
    pub otherwise: Option<ElseNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum ElseNode<'a> {
    If(Box<IfNode<'a>>),
    Block(BlockNode<'a>),
}
