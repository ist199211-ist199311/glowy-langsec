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
