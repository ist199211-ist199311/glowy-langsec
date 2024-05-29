use std::{cmp::Ordering, collections::BTreeSet, fmt::Display, ops::BitOr};

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

    // TODO support saying if changed?
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
