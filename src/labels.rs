use std::{cmp::Ordering, collections::BTreeSet, fmt::Display};

use parser::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Label<'a> {
    Top,
    // This must never have an empty set (use Bottom instead)
    Parts(BTreeSet<&'a str>),
    Bottom,
}

impl<'a> Label<'a> {
    pub fn from_parts(parts: &[&'a str]) -> Label<'a> {
        if parts.is_empty() {
            Label::Bottom
        } else {
            let parts = BTreeSet::from_iter(parts.iter().cloned());
            Label::Parts(parts)
        }
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
            (Self::Parts(lparts), Self::Parts(rparts)) => Self::Parts(lparts & rparts),
            (Self::Bottom, Self::Bottom) => Self::Bottom,
        }
    }

    pub fn difference(&self, other: &Label<'a>) -> Label<'a> {
        match (self, other) {
            (_, Self::Top) => Self::Bottom,
            (Self::Top, _) => Self::Top,
            (Self::Parts(_), Self::Bottom) => self.clone(),
            (Self::Parts(lparts), Self::Parts(rparts)) => {
                Self::Parts(lparts.difference(rparts).cloned().collect())
            }
            (Self::Bottom, _) => Self::Bottom,
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

/// Keeps track of where labels come from.
/// This represents a tree, where each node has children indicating where
/// the labels come from.
/// The following assumptions are strictly enforced:
/// - Two distinct children cannot share any label;
/// - Labels of children are always a subset of their parent's label.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelBacktrace<'a> {
    r#type: LabelBacktraceType,
    file_id: usize,
    symbol: Span<'a>,
    label: Label<'a>,
    children: Vec<LabelBacktrace<'a>>,
}

impl<'a> LabelBacktrace<'a> {
    pub fn new_explicit_annotation(file_id: usize, symbol: Span<'a>, label: Label<'a>) -> Self {
        Self {
            r#type: LabelBacktraceType::ExplicitAnnotation,
            file_id,
            symbol,
            label,
            children: vec![],
        }
    }

    pub fn new<'b>(
        r#type: LabelBacktraceType,
        file_id: usize,
        symbol: Span<'a>,
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
                        let child = Self::restrict_to_label(&remaining_label, child);
                        if let Some(child) = &child {
                            remaining_label = remaining_label.difference(child.label());
                        }
                        child
                    })
                    .collect()
            }
        };

        if children.len() == 1
            && children.first().unwrap().label() == &label
            && children.first().unwrap().symbol() == &symbol
        {
            // avoid multiple repeated backtraces to the same symbol
            children.first().cloned()
        } else {
            Some(LabelBacktrace {
                r#type,
                file_id,
                symbol,
                label,
                children,
            })
        }
    }

    /// Ensure this LabelBacktrace only mentions the provided label,
    /// pruning children if they have label bottom.
    fn restrict_to_label(
        label: &Label<'a>,
        backtrace: &LabelBacktrace<'a>,
    ) -> Option<LabelBacktrace<'a>> {
        let new_label = label.intersect(backtrace.label());
        match new_label {
            Label::Bottom => None,
            new_label => Some(Self {
                r#type: backtrace.r#type,
                file_id: backtrace.file_id,
                symbol: backtrace.symbol.clone(),
                label: new_label,
                children: backtrace
                    .children
                    .iter()
                    .filter_map(|child| Self::restrict_to_label(label, child))
                    .collect(),
            }),
        }
    }

    pub fn label(&self) -> &Label<'a> {
        &self.label
    }

    pub fn symbol(&self) -> &Span<'a> {
        &self.symbol
    }

    pub fn file(&self) -> usize {
        self.file_id
    }

    pub fn r#type(&self) -> LabelBacktraceType {
        self.r#type
    }

    pub fn children(&self) -> &[LabelBacktrace<'a>] {
        &self.children
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelBacktraceType {
    ExplicitAnnotation,
    Assignment,
    Expression,
    Branch,
    FunctionCall,
    Return,
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::BTreeSet};

    use super::Label;

    #[test]
    fn label_constructor() {
        assert_eq!(Label::from_parts(&[]), Label::Bottom);
        assert_eq!(
            Label::from_parts(&["lbl1", "lbl2"]),
            Label::Parts(BTreeSet::from(["lbl1", "lbl2"]))
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
}
