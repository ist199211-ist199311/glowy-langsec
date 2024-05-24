use crate::Span;

#[derive(Debug, PartialEq)]
pub struct SourceFileNode<'a> {
    pub package_clause: PackageClauseNode<'a>,
    // TODO: pub imports: Vec<TopLevelDeclNode<'a>>,
    pub top_level_decls: Vec<TopLevelDeclNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct PackageClauseNode<'a> {
    pub id: Span<'a>,
}

#[derive(Debug, PartialEq)]
pub enum TopLevelDeclNode<'a> {
    Const(Vec<BindingDeclSpecNode<'a>>),
    Var(Vec<BindingDeclSpecNode<'a>>),
    Function(FunctionDeclNode<'a>),
}

impl<'a> From<FunctionDeclNode<'a>> for TopLevelDeclNode<'a> {
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
    // TODO: Literal
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
    pub result: FunctionResultNode<'a>,
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

#[derive(Debug, PartialEq)]
pub struct OperandNameNode<'a> {
    pub package: Option<Span<'a>>, // for qualified operand names
    pub id: Span<'a>,
}

#[derive(Debug, PartialEq)]
pub enum LiteralNode {
    Int(u64),
}

pub type BlockNode<'a> = Vec<StatementNode<'a>>;

#[derive(Debug, PartialEq)]
pub enum StatementNode<'a> {
    // TODO: support statements
    Placeholder(Span<'a>),
}
