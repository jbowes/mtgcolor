//! Comparison operators shared by color sets and mana costs.

use core::cmp::Ordering;

/// A Scryfall comparison operator (`=`, `!=`, `<`, `<=`, `>`, `>=`).
///
/// On colors and mana costs the order is *containment*, not magnitude: `a <= b` means `a` is a
/// subset of `b`. Two values can be incomparable, in which case only `!=` holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cmp {
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

impl Cmp {
    /// Parses `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`. Scryfall's `:` is context dependent and is
    /// not handled here.
    #[must_use]
    pub fn parse(op: &str) -> Option<Self> {
        match op {
            "=" | "==" => Some(Self::Eq),
            "!=" => Some(Self::Ne),
            "<" => Some(Self::Lt),
            "<=" => Some(Self::Le),
            ">" => Some(Self::Gt),
            ">=" => Some(Self::Ge),
            _ => None,
        }
    }

    /// The operator as written in a Scryfall query.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }

    /// Whether the operator holds for the result of a partial comparison of `lhs` to `rhs`.
    /// `None` means the two were incomparable.
    #[must_use]
    pub const fn holds(self, ordering: Option<Ordering>) -> bool {
        match ordering {
            Some(Ordering::Less) => matches!(self, Self::Lt | Self::Le | Self::Ne),
            Some(Ordering::Equal) => matches!(self, Self::Eq | Self::Le | Self::Ge),
            Some(Ordering::Greater) => matches!(self, Self::Gt | Self::Ge | Self::Ne),
            None => matches!(self, Self::Ne),
        }
    }
}
