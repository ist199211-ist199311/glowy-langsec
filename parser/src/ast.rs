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
    Const(Vec<ConstDeclSpecNode<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct ConstDeclSpecNode<'a> {
    pub ids: Vec<Span<'a>>,
    // TODO: pub r#type: Option<___>
    // TODO: pub exprs: Vec<ExprNode>
}

#[derive(Debug, PartialEq)]
pub enum ExprNode<'a> {
    Name(OperandNameNode<'a>),
    Literal(LiteralNode),
    // TODO: more primary expressions...
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
    Float(f64),
    Imaginary(f64),
    Rune(i32),
    String(String),
}
