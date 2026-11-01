//! Color identity, the Commander deck-building rule.

use core::fmt;
use core::str::FromStr;

use crate::cmp::Cmp;
use crate::color::{Color, ColorSet};
use crate::cost::ManaCost;
use crate::error::ParseError;
use crate::symbol::{Symbol, Symbols};

/// A card's color identity: the colors a Commander deck containing it must be able to afford.
///
/// This is a [`ColorSet`] with a distinct type, so a card's *color* and its *identity* cannot be
/// mixed up. Build one by combining the parts a card has with [`union`](Self::union):
///
/// ```
/// use mtgcolor::{ColorIdentity, ManaCost};
///
/// let cost: ManaCost = "{2}{G}{U}".parse().unwrap();
/// let identity = ColorIdentity::from_cost(&cost)
///     | ColorIdentity::from_oracle_text("{T}: Add {R}. (Reminder: add {W}.)")
///     | ColorIdentity::from_type_line("Legendary Land");
/// assert_eq!(identity.to_string(), "URG");
/// ```
///
/// The parts follow the Comprehensive Rules (903.4): the colors of symbols in the cost and rules
/// text, a color indicator, and the basic land types. Color indicators are not text, so pass them
/// as a [`ColorSet`] through [`new`](Self::new).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorIdentity(ColorSet);

impl ColorIdentity {
    /// No colors.
    pub const COLORLESS: Self = Self(ColorSet::COLORLESS);

    /// Wraps a set of colors.
    #[must_use]
    pub const fn new(colors: ColorSet) -> Self {
        Self(colors)
    }

    /// The colors, as a plain set.
    #[must_use]
    pub const fn colors(self) -> ColorSet {
        self.0
    }

    /// The identity contributed by a mana cost.
    #[must_use]
    pub fn from_cost(cost: &ManaCost) -> Self {
        Self(cost.colors())
    }

    /// The identity contributed by mana symbols in Oracle text. Reminder text, which is in
    /// parentheses, is ignored. Symbols this crate does not recognise are ignored too.
    #[must_use]
    pub fn from_oracle_text(text: &str) -> Self {
        let colors = Symbols::outside_reminder_text(text)
            .filter_map(Result::ok)
            .filter_map(Symbol::pip)
            .fold(ColorSet::COLORLESS, |set, pip| set | pip.colors());
        Self(colors)
    }

    /// The identity contributed by basic land types on a type line (`Basic Land — Forest`):
    /// each one gives the color it taps for.
    #[must_use]
    pub fn from_type_line(type_line: &str) -> Self {
        let colors = Color::ALL
            .into_iter()
            .filter(|color| {
                type_line
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|word| word == color.basic_land_type())
            })
            .collect();
        Self(colors)
    }

    /// Both identities combined.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0.union(other.0))
    }

    /// Whether this is colorless.
    #[must_use]
    pub const fn is_colorless(self) -> bool {
        self.0.is_empty()
    }

    /// Whether a deck led by `commander` may contain a card with this identity: every color here
    /// is also in the commander's identity (Scryfall's `id<=`).
    #[must_use]
    pub const fn fits_in(self, commander: Self) -> bool {
        self.0.is_subset(commander.0)
    }

    /// Applies a Scryfall comparison by containment (`<=` is "fits in").
    #[must_use]
    pub const fn compare(self, op: Cmp, other: Self) -> bool {
        self.0.compare(op, other.0)
    }

    /// Parses Scryfall's color-identity syntax: see [`ColorSet::from_scryfall`].
    ///
    /// # Errors
    ///
    /// As [`ColorSet::from_scryfall`].
    pub fn from_scryfall(s: &str) -> Result<Self, ParseError> {
        ColorSet::from_scryfall(s).map(Self)
    }

    /// The one-byte binary form: see [`ColorSet::to_byte`].
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        self.0.to_byte()
    }

    /// Reads the one-byte binary form.
    ///
    /// # Errors
    ///
    /// [`DecodeError::InvalidEncoding`](crate::DecodeError::InvalidEncoding) if any of the top
    /// three bits are set.
    pub const fn from_byte(byte: u8) -> Result<Self, crate::DecodeError> {
        match ColorSet::from_byte(byte) {
            Ok(colors) => Ok(Self(colors)),
            Err(e) => Err(e),
        }
    }
}

impl fmt::Debug for ColorIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColorIdentity({})", self.0)
    }
}

impl fmt::Display for ColorIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for ColorIdentity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_scryfall(s)
    }
}

impl From<ColorSet> for ColorIdentity {
    fn from(colors: ColorSet) -> Self {
        Self(colors)
    }
}

impl From<ColorIdentity> for ColorSet {
    fn from(identity: ColorIdentity) -> Self {
        identity.0
    }
}

impl core::ops::BitOr for ColorIdentity {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}
