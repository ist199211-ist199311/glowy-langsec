use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

use parser::{Location, Span};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LabelTag<'a> {
    Concrete(&'a str),
    Synthetic(usize),
}

impl<'a> Display for LabelTag<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelTag::Concrete(tag) => write!(f, "{}", tag),
            LabelTag::Synthetic(id) => write!(f, "<{}>", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Label<'a> {
    Top,
    // This must never have an empty set (use Bottom instead)
    Parts(BTreeSet<LabelTag<'a>>),
    Bottom,
}

impl<'a> Label<'a> {
    pub fn from_parts(parts: &[&'a str]) -> Label<'a> {
        if parts.is_empty() {
            Label::Bottom
        } else {
            let parts = BTreeSet::from_iter(parts.iter().map(|part| LabelTag::Concrete(part)));
            Label::Parts(parts)
        }
    }

    pub fn from_synthetic_id(id: usize) -> Label<'a> {
        Label::Parts(BTreeSet::from([LabelTag::Synthetic(id)]))
    }

    pub fn union(&self, other: &Label<'a>) -> Label<'a> {
        match (self, other) {
            (Self::Top, _) => Self::Top,
            (_, Self::Top) => Self::Top,
            (Self::Parts(_), Self::Bottom) => self.clone(),
            (Self::Bottom, Self::Parts(_)) => other.clone(),
            (Self::Parts(lparts), Self::Parts(rparts)) => Self::Parts(lparts | rparts),
            (Self::Bottom, Self::Bottom) => Self::Bottom,
        }
    }

    pub fn intersect(&self, other: &Label<'a>) -> Label<'a> {
        match (self, other) {
            (Self::Top, _) => other.clone(),
            (_, Self::Top) => self.clone(),
            (Self::Parts(_), Self::Bottom) => Self::Bottom,
            (Self::Bottom, Self::Parts(_)) => Self::Bottom,
            (Self::Parts(lparts), Self::Parts(rparts)) => {
                let new_parts = lparts & rparts;
                if new_parts.is_empty() {
                    Self::Bottom
                } else {
                    Self::Parts(new_parts)
                }
            }
            (Self::Bottom, Self::Bottom) => Self::Bottom,
        }
    }

    pub fn difference(&self, other: &Label<'a>) -> Label<'a> {
        match (self, other) {
            (_, Self::Top) => Self::Bottom,
            (Self::Top, _) => Self::Top,
            (Self::Parts(_), Self::Bottom) => self.clone(),
            (Self::Parts(lparts), Self::Parts(rparts)) => {
                let new_parts = lparts - rparts;
                if new_parts.is_empty() {
                    Self::Bottom
                } else {
                    Self::Parts(new_parts)
                }
            }
            (Self::Bottom, _) => Self::Bottom,
        }
    }

    pub fn replace_synthetic_tags(&self, replacements: &[Label<'a>]) -> Label<'a> {
        if let Label::Parts(parts) = self {
            let mut concrete_parts = BTreeSet::new();
            let mut synthetic_parts = vec![];

            for part in parts {
                if let LabelTag::Synthetic(id) = part {
                    if *id < replacements.len() {
                        synthetic_parts.push(*id);
                        continue;
                    }
                }
                concrete_parts.insert(part.clone());
            }

            synthetic_parts
                .iter()
                .fold(Label::Bottom, |acc, id| {
                    // id has been checked to be within the slice's length
                    acc.union(&replacements[*id])
                })
                .union(&Label::Parts(concrete_parts))
        } else {
            self.clone()
        }
    }
}

impl<'a> PartialOrd for Label<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        match (self, other) {
            (Self::Top, _) => Some(Ordering::Greater),
            (_, Self::Top) => Some(Ordering::Less),
            (Self::Bottom, _) => Some(Ordering::Less),
            (_, Self::Bottom) => Some(Ordering::Greater),
            (Self::Parts(lparts), Self::Parts(rparts)) => {
                if rparts.is_subset(lparts) {
                    Some(Ordering::Greater)
                } else if lparts.is_subset(rparts) {
                    Some(Ordering::Less)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a> Display for Label<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Top => write!(f, "<top>"),
            Label::Parts(parts) => {
                write!(f, "{{")?;
                let mut iter = parts.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{}", first)?;
                    for part in iter {
                        write!(f, ", {}", part)?;
                    }
                }
                write!(f, "}}")
            }
            Label::Bottom => write!(f, "{{}}"),
        }
    }
}

impl<'a> From<&Option<LabelBacktrace<'a>>> for Label<'a> {
    fn from(opt: &Option<LabelBacktrace<'a>>) -> Self {
        opt.as_ref()
            .map(LabelBacktrace::label)
            .cloned()
            .unwrap_or(Label::Bottom)
    }
}

/// Keeps track of where labels come from.
/// This represents a tree, where each node has children indicating where
/// the labels come from.
/// The following assumptions are strictly enforced:
/// - Two distinct children cannot share any label;
/// - Labels of children are always a subset of their parent's label.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelBacktrace<'a> {
    kind: LabelBacktraceKind,
    file_id: usize,
    location: Location,
    symbol: Option<Span<'a>>,
    label: Label<'a>,
    children: Vec<LabelBacktrace<'a>>,
}

impl<'a> LabelBacktrace<'a> {
    pub fn new_explicit_annotation(file_id: usize, symbol: Span<'a>, label: Label<'a>) -> Self {
        Self {
            kind: LabelBacktraceKind::ExplicitAnnotation,
            file_id,
            location: symbol.location(),
            symbol: Some(symbol),
            label,
            children: vec![],
        }
    }

    pub fn new<'b>(
        kind: LabelBacktraceKind,
        file_id: usize,
        location: Location,
        symbol: Option<Span<'a>>,
        label: Label<'a>,
        children: impl IntoIterator<Item = &'b LabelBacktrace<'a>>,
    ) -> Option<Self>
    where
        'a: 'b,
    {
        let children: Vec<_> = match label {
            Label::Bottom => return None,
            Label::Top => children
                .into_iter()
                .find(|child| child.label() == &Label::Top)
                .into_iter()
                .cloned()
                .collect(),
            Label::Parts(_) => {
                let mut remaining_label = label.clone();
                children
                    .into_iter()
                    .filter_map(|child| {
                        let child = child.restrict_to_label(&remaining_label);
                        if let Some(child) = &child {
                            remaining_label = remaining_label.difference(child.label());
                        }
                        child
                    })
                    .collect()
            }
        };

        // if there is only one child
        if let [child] = children.as_slice() {
            if child.label == label && child.location == location && child.symbol == symbol {
                // avoid multiple repeated backtraces to the same symbol
                return Some(child.clone());
            }
        };

        Some(LabelBacktrace {
            kind,
            file_id,
            location,
            symbol,
            label,
            children,
        })
    }

    /// Ensure this LabelBacktrace only mentions the provided label,
    /// pruning children if they have label bottom.
    fn restrict_to_label(&self, label: &Label<'a>) -> Option<LabelBacktrace<'a>> {
        let new_label = self.label.intersect(label);

        if new_label == Label::Bottom {
            None
        } else {
            Some(Self {
                kind: self.kind,
                file_id: self.file_id,
                location: self.location.clone(),
                symbol: self.symbol.clone(),
                label: new_label,
                children: self
                    .children
                    .iter()
                    .filter_map(|child| child.restrict_to_label(label))
                    .collect(),
            })
        }
    }

    pub fn from_children<'b>(
        children: impl Iterator<Item = &'b Option<LabelBacktrace<'a>>>,
        with_kind: LabelBacktraceKind,
        file_id: usize,
        at_location: Location,
        symbol: Option<Span<'a>>,
    ) -> Option<LabelBacktrace<'a>>
    where
        'a: 'b,
    {
        let backtraces: Vec<&LabelBacktrace<'a>> =
            children.filter(|x| Option::is_some(x)).flatten().collect();

        let label = backtraces
            .iter()
            .map(|bt| bt.label())
            .fold(Label::Bottom, |acc, label| acc.union(label));

        Self::new(with_kind, file_id, at_location, symbol, label, backtraces)
    }

    pub fn with_child(&self, child: &LabelBacktrace<'a>) -> LabelBacktrace<'a> {
        Self::new(
            self.kind,
            self.file_id,
            self.location.clone(),
            self.symbol.clone(),
            self.label.union(child.label()),
            std::iter::once(child).chain(self.children.iter()),
        )
        .unwrap() // safe because if self exists, label is not Bottom
    }

    pub fn union(
        &self,
        other: &LabelBacktrace<'a>,
        with_kind: LabelBacktraceKind,
        at_location: Location,
        symbol: Option<Span<'a>>,
    ) -> LabelBacktrace<'a> {
        Self::new(
            with_kind,
            self.file_id, // FIXME: this is not necessarily true...
            at_location,
            symbol,
            self.label.union(other.label()),
            [self, other],
        )
        .unwrap() // safe because if self exists, label is not Bottom
    }

    pub fn replace_synthetic_tags(self, replacements: &[Label<'a>]) -> Option<Self> {
        let new_label = self.label.replace_synthetic_tags(replacements);

        if new_label == Label::Bottom {
            None
        } else {
            Some(Self {
                kind: self.kind,
                file_id: self.file_id,
                location: self.location,
                symbol: self.symbol,
                label: new_label,
                children: self
                    .children
                    .into_iter()
                    .filter_map(|child| child.replace_synthetic_tags(replacements))
                    .collect(),
            })
        }
    }

    pub fn label(&self) -> &Label<'a> {
        &self.label
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn symbol(&self) -> &Option<Span<'a>> {
        &self.symbol
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn kind(&self) -> LabelBacktraceKind {
        self.kind
    }

    pub fn children(&self) -> &[LabelBacktrace<'a>] {
        &self.children
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelBacktraceKind {
    ExplicitAnnotation,
    Assignment,
    Expression,
    Branch,
    FunctionArgument,
    FunctionCall,
    Return,
    Send,
    Receive,
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::BTreeSet};

    use super::{Label, LabelTag};

    #[test]
    fn label_constructor() {
        assert_eq!(Label::from_parts(&[]), Label::Bottom);
        assert_eq!(
            Label::from_parts(&["lbl1", "lbl2"]),
            Label::Parts(BTreeSet::from([
                LabelTag::Concrete("lbl1"),
                LabelTag::Concrete("lbl2")
            ]))
        );
    }

    #[test]
    fn label_union() {
        macro_rules! union {
            ($left: expr, $right: expr, $expected: expr) => {
                assert_eq!($left.union(&$right), $expected);
                assert_eq!($right.union(&$left), $expected);
            };
        }

        union!(Label::Top, Label::Top, Label::Top);
        union!(Label::Top, Label::from_parts(&["lbl1"]), Label::Top);
        union!(Label::Top, Label::Bottom, Label::Top);

        union!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::from_parts(&["lbl2", "lbl3"]),
            Label::from_parts(&["lbl1", "lbl2", "lbl3"])
        );
        union!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::Bottom,
            Label::from_parts(&["lbl1", "lbl3"])
        );

        union!(Label::Bottom, Label::Bottom, Label::Bottom);
    }

    #[test]
    fn label_intersect() {
        macro_rules! intersect {
            ($left: expr, $right: expr, $expected: expr) => {
                assert_eq!($left.intersect(&$right), $expected);
                assert_eq!($right.intersect(&$left), $expected);
            };
        }

        intersect!(Label::Top, Label::Top, Label::Top);
        intersect!(
            Label::Top,
            Label::from_parts(&["lbl1"]),
            Label::from_parts(&["lbl1"])
        );
        intersect!(Label::Top, Label::Bottom, Label::Bottom);

        intersect!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::from_parts(&["lbl2", "lbl3"]),
            Label::from_parts(&["lbl3"])
        );
        intersect!(
            Label::from_parts(&["lbl1", "lbl2"]),
            Label::from_parts(&["lbl3", "lbl4"]),
            Label::Bottom
        );
        intersect!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::Bottom,
            Label::Bottom
        );

        intersect!(Label::Bottom, Label::Bottom, Label::Bottom);
    }

    #[test]
    fn label_difference() {
        macro_rules! difference {
            ($left: expr, $right: expr, $expected_lr: expr, $expected_rl: expr) => {
                assert_eq!($left.difference(&$right), $expected_lr);
                assert_eq!($right.difference(&$left), $expected_rl);
            };
        }

        difference!(Label::Top, Label::Top, Label::Bottom, Label::Bottom);
        difference!(
            Label::Top,
            Label::from_parts(&["lbl1"]),
            Label::Top,
            Label::Bottom
        );
        difference!(Label::Top, Label::Bottom, Label::Top, Label::Bottom);

        difference!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::from_parts(&["lbl2", "lbl3"]),
            Label::from_parts(&["lbl1"]),
            Label::from_parts(&["lbl2"])
        );
        difference!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::Bottom,
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::Bottom
        );
        difference!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::Bottom,
            Label::Bottom
        );

        difference!(Label::Bottom, Label::Bottom, Label::Bottom, Label::Bottom);
    }

    #[test]
    fn compare_labels() {
        macro_rules! cmp {
            ($left: expr, $right: expr, $expected: expr) => {
                assert_eq!($left.partial_cmp(&$right), $expected);
            };
        }

        cmp!(Label::Bottom, Label::Bottom, Some(Ordering::Equal));
        cmp!(Label::Top, Label::Top, Some(Ordering::Equal));
        cmp!(Label::Bottom, Label::Top, Some(Ordering::Less));
        cmp!(Label::Top, Label::Bottom, Some(Ordering::Greater));

        cmp!(
            Label::Top,
            Label::from_parts(&["lbl1"]),
            Some(Ordering::Greater)
        );
        cmp!(
            Label::from_parts(&["lbl1"]),
            Label::Top,
            Some(Ordering::Less)
        );
        cmp!(
            Label::Bottom,
            Label::from_parts(&["lbl1"]),
            Some(Ordering::Less)
        );
        cmp!(
            Label::from_parts(&["lbl1"]),
            Label::Bottom,
            Some(Ordering::Greater)
        );

        cmp!(
            Label::from_parts(&["lbl1"]),
            Label::from_parts(&["lbl2"]),
            None
        );
        cmp!(
            Label::from_parts(&["lbl2"]),
            Label::from_parts(&["lbl1"]),
            None
        );
        cmp!(
            Label::from_parts(&["lbl1", "lbl2"]),
            Label::from_parts(&["lbl2"]),
            Some(Ordering::Greater)
        );
        cmp!(
            Label::from_parts(&["lbl1"]),
            Label::from_parts(&["lbl1", "lbl2"]),
            Some(Ordering::Less)
        );
        cmp!(
            Label::from_parts(&["lbl1", "lbl3"]),
            Label::from_parts(&["lbl1", "lbl2"]),
            None
        );
        cmp!(
            Label::from_parts(&["lbl1", "lbl2"]),
            Label::from_parts(&["lbl1", "lbl3"]),
            None
        );
    }

    #[test]
    fn replace_synthetic_tags() {
        let label_with_synthetic_parts = Label::Parts(BTreeSet::from([
            LabelTag::Concrete("lbl1"),
            LabelTag::Synthetic(0),
            LabelTag::Synthetic(1),
        ]));
        let replacements = [
            Label::Parts(BTreeSet::from([
                LabelTag::Concrete("lbl1"),
                LabelTag::Concrete("lbl2"),
                LabelTag::Synthetic(0),
            ])),
            Label::Parts(BTreeSet::from([
                LabelTag::Concrete("lbl2"),
                LabelTag::Concrete("lbl3"),
            ])),
        ];

        let expected = Label::Parts(BTreeSet::from([
            LabelTag::Concrete("lbl1"),
            LabelTag::Concrete("lbl2"),
            LabelTag::Concrete("lbl3"),
            LabelTag::Synthetic(0),
        ]));

        assert_eq!(
            label_with_synthetic_parts.replace_synthetic_tags(&replacements),
            expected
        );
    }
}
