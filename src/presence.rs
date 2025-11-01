use std::{fmt, slice::SliceIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence<T> {
    Absent,
    Null,
    Some(T),
}

impl<T> Presence<T> {
    pub fn absent(&self) -> Self {
        matches!(self, Presence::Absent)
    }

    pub fn null(&self) -> Self {
        matches!(self, Presence::Null)
    }

    pub fn ok(&self) -> Self {
        matches!(self, Presence::Some(_))
    }

    pub fn as_ref(&self) -> &T {
        match self {
            Presence::Some(val) => Some(val),
            _ => None,
        }
    }
}
