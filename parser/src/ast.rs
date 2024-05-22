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
    pub mapping: Vec<(Span<'a>, ExprNode<'a>)>,
    // TODO: pub r#type: Option<___>
}

#[derive(Debug)]
pub struct MismatchingConstDeclSpecListsLength;

impl<'a> ConstDeclSpecNode<'a> {
    pub fn try_new(
        ids: Vec<Span<'a>>,
        exprs: Vec<ExprNode<'a>>,
    ) -> Result<Self, MismatchingConstDeclSpecListsLength> {
        if ids.len() != exprs.len() {
            Err(MismatchingConstDeclSpecListsLength {})
        } else {
            Ok(Self {
                mapping: ids.into_iter().zip(exprs).collect(),
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExprNode<'a> {
    Name(OperandNameNode<'a>),
    Literal(LiteralNode),
    // TODO: more expressions...
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
