use std::{cmp::Ordering, collections::BTreeSet, ops::BitOr};

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
