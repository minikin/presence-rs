use std::{fmt, slice::SliceIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence<T> {
    /// Field/key is absent from the structure
    Absent,
    /// Field/key is present but the value is null
    Null,
    /// Field/key is present with a concrete value
    Some(T),
}

impl<T> Presence<T> {
    /// Check if the field is absent
    pub fn is_absent(&self) -> bool {
        matches!(self, Presence::Absent)
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Presence::Null)
    }

    /// Check if a concrete value is present
    pub fn is_present(&self) -> bool {
        matches!(self, Presence::Some(_))
    }

    /// Get a reference to the value if present
    pub fn as_ref(&self) -> &T {
        match *self {
            Presence::Some(val) => Some(val),
            _ => None,
        }
    }

    /// Get a mutable reference to the value if present
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match *self {
            Presence::Some(val) => Some(val),
            _ => None,
        }
    }

    /// Convert to Option<Option<T>> for interop
    pub fn to_nested_option(self) -> Option<Option<T>> {
        match self {
            Presence::Absent => None,
            Presence::Null => Some(None),
            Presence::Some(val) => Some(Some(val)),
        }
    }
}
