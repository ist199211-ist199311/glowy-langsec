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
    // TODO: implement actual expressions
    Placeholder(Span<'a>),
}
