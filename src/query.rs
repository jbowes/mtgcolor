//! The value side of Scryfall's `c:` and `id:` keywords.

use crate::cmp::Cmp;
use crate::color::ColorSet;
use crate::error::ParseError;
use alloc::string::ToString;

/// Which keyword a color value was written for. Scryfall's bare `:` means different things for
/// each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorKind {
    /// `c:` / `color:`, a card's colors.
    Color,
    /// `id:` / `identity:`, a card's color identity.
    Identity,
}

/// What follows `c:` or `id:`: a set of colors, the word `multicolor`, or a count.
///
/// ```
/// use mtgcolor::{ColorKind, ColorSet, ColorSpec};
///
/// let spec = ColorSpec::parse("esper").unwrap();
/// // `id:esper` includes any identity that fits in Esper.
/// assert!(spec.matches_colon(ColorKind::Identity, "WU".parse().unwrap()));
/// // `c:esper` wants at least white, blue and black.
/// assert!(!spec.matches_colon(ColorKind::Color, "WU".parse().unwrap()));
/// assert!(ColorSpec::parse("2").unwrap().matches_colon(ColorKind::Color, "WU".parse().unwrap()));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorSpec {
    /// A color expression: `rg`, `azorius`, `c`.
    Set(ColorSet),
    /// `m` or `multicolor`: two or more colors.
    Multicolor,
    /// A number: that many colors.
    Count(u8),
}

impl ColorSpec {
    /// Parses a `c:`/`id:` value: anything [`ColorSet::from_scryfall`] takes, plus `m`,
    /// `multicolor`, and the digits `0` to `5`.
    ///
    /// # Errors
    ///
    /// [`ParseError::InvalidColorCount`] for a number above 5, otherwise as
    /// [`ColorSet::from_scryfall`].
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("m") || s.eq_ignore_ascii_case("multicolor") {
            return Ok(Self::Multicolor);
        }
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
            return match s.parse::<u8>() {
                Ok(n) if n <= 5 => Ok(Self::Count(n)),
                _ => Err(ParseError::InvalidColorCount(s.to_string())),
            };
        }
        ColorSet::from_scryfall(s).map(Self::Set)
    }

    /// Whether `actual` satisfies `actual <op> self`.
    ///
    /// For a color set the comparison is by containment. For a count it compares the number of
    /// colors. [`Multicolor`](Self::Multicolor) means "two or more colors": `=` and `>=` hold
    /// when `actual` is multicolor, `!=` when it is not, and the other operators never hold.
    #[must_use]
    pub fn matches(self, op: Cmp, actual: ColorSet) -> bool {
        match self {
            Self::Set(set) => actual.compare(op, set),
            Self::Count(n) => op.holds(Some(actual.len().cmp(&u32::from(n)))),
            Self::Multicolor => match op {
                Cmp::Eq | Cmp::Ge => actual.is_multicolor(),
                Cmp::Ne => !actual.is_multicolor(),
                Cmp::Lt | Cmp::Le | Cmp::Gt => false,
            },
        }
    }

    /// Whether `actual` satisfies the bare `:` form of the keyword.
    ///
    /// `c:rg` means "at least red and green" (`>=`), `id:rg` means "fits in red-green" (`<=`),
    /// and a colorless value, a count or `multicolor` means equality.
    #[must_use]
    pub fn matches_colon(self, kind: ColorKind, actual: ColorSet) -> bool {
        let op = match self {
            Self::Set(set) if !set.is_empty() => match kind {
                ColorKind::Color => Cmp::Ge,
                ColorKind::Identity => Cmp::Le,
            },
            _ => Cmp::Eq,
        };
        self.matches(op, actual)
    }
}
