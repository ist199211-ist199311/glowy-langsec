use crate::Span;

#[derive(Debug, PartialEq)]
pub struct Node<'a> {
    kind: NodeKind<'a>,
}

impl<'a> Node<'a> {
    pub fn new(kind: NodeKind<'a>) -> Self {
        Self { kind }
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeKind<'a> {
    PackageClause { id: Span<'a> },
}
