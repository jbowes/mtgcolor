//! The five colors and sets of them.

use alloc::string::ToString;
use core::cmp::Ordering;
use core::fmt;
use core::iter::FusedIterator;
use core::ops::{BitAnd, BitOr, BitXor, Not, Sub};
use core::str::FromStr;

use crate::cmp::Cmp;
use crate::error::{DecodeError, ParseError};

/// One of the five colors of Magic, in WUBRG order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    /// White (`W`)
    White = 0,
    /// Blue (`U`)
    Blue = 1,
    /// Black (`B`)
    Black = 2,
    /// Red (`R`)
    Red = 3,
    /// Green (`G`)
    Green = 4,
}

impl Color {
    /// All five colors in WUBRG order.
    pub const ALL: [Self; 5] = [Self::White, Self::Blue, Self::Black, Self::Red, Self::Green];

    /// All five colors in alphabetical order of their letters: B, G, R, U, W. This is the order
    /// Scryfall and MTGJSON use when they list colors, which is why serialization uses it.
    pub const ALPHABETICAL: [Self; 5] =
        [Self::Black, Self::Green, Self::Red, Self::Blue, Self::White];

    /// Position in WUBRG order, `0..=4`.
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The color at a WUBRG position, if `index < 5`.
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::White),
            1 => Some(Self::Blue),
            2 => Some(Self::Black),
            3 => Some(Self::Red),
            4 => Some(Self::Green),
            _ => None,
        }
    }

    /// The single bit this color occupies in a [`ColorSet`].
    #[must_use]
    pub const fn bit(self) -> u8 {
        1 << (self as u8)
    }

    /// The uppercase letter: `W`, `U`, `B`, `R` or `G`.
    #[must_use]
    pub const fn letter(self) -> char {
        match self {
            Self::White => 'W',
            Self::Blue => 'U',
            Self::Black => 'B',
            Self::Red => 'R',
            Self::Green => 'G',
        }
    }

    /// Parses a color letter, in either case.
    #[must_use]
    pub const fn from_letter(letter: char) -> Option<Self> {
        match letter {
            'W' | 'w' => Some(Self::White),
            'U' | 'u' => Some(Self::Blue),
            'B' | 'b' => Some(Self::Black),
            'R' | 'r' => Some(Self::Red),
            'G' | 'g' => Some(Self::Green),
            _ => None,
        }
    }

    /// The lowercase English name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Blue => "blue",
            Self::Black => "black",
            Self::Red => "red",
            Self::Green => "green",
        }
    }

    /// Parses a color name such as `blue`, in any case.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|c| name.eq_ignore_ascii_case(c.name()))
    }

    /// The basic land type that taps for this color.
    #[must_use]
    pub const fn basic_land_type(self) -> &'static str {
        match self {
            Self::White => "Plains",
            Self::Blue => "Island",
            Self::Black => "Swamp",
            Self::Red => "Mountain",
            Self::Green => "Forest",
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Write::write_char(f, self.letter())
    }
}

impl FromStr for Color {
    type Err = ParseError;

    /// Accepts a letter (`u`) or a name (`blue`), in any case.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if let Some(color) = Self::from_letter(c) {
                return Ok(color);
            }
        }
        Self::from_name(s).ok_or_else(|| ParseError::UnknownColor(s.to_string()))
    }
}

/// A set of colors: any of the 32 combinations of W, U, B, R and G, stored in one byte.
///
/// Bit 0 is white, then blue, black, red and green. The empty set is colorless. Iteration,
/// [`Display`](fmt::Display) and the serialized forms all use WUBRG order.
///
/// `Ord` compares the raw bits and exists so sets can key maps; it is *not* the subset order. Use
/// [`subset_cmp`](Self::subset_cmp) or [`compare`](Self::compare) for that.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorSet(u8);

/// Names Scryfall accepts for color sets (`c:azorius`, `id<=esper`). The first match wins when
/// naming a set, so guilds come before the colleges that share their colors.
const NAMED: [(&str, ColorSet); 30] = [
    ("white", ColorSet(0b00001)),
    ("blue", ColorSet(0b00010)),
    ("black", ColorSet(0b00100)),
    ("red", ColorSet(0b01000)),
    ("green", ColorSet(0b10000)),
    // Ravnica guilds.
    ("azorius", ColorSet(0b00011)),
    ("dimir", ColorSet(0b00110)),
    ("rakdos", ColorSet(0b01100)),
    ("gruul", ColorSet(0b11000)),
    ("selesnya", ColorSet(0b10001)),
    ("orzhov", ColorSet(0b00101)),
    ("izzet", ColorSet(0b01010)),
    ("golgari", ColorSet(0b10100)),
    ("boros", ColorSet(0b01001)),
    ("simic", ColorSet(0b10010)),
    // Strixhaven colleges (same colors as five of the guilds).
    ("silverquill", ColorSet(0b00101)),
    ("prismari", ColorSet(0b01010)),
    ("witherbloom", ColorSet(0b10100)),
    ("lorehold", ColorSet(0b01001)),
    ("quandrix", ColorSet(0b10010)),
    // Alara shards.
    ("bant", ColorSet(0b10011)),
    ("esper", ColorSet(0b00111)),
    ("grixis", ColorSet(0b01110)),
    ("jund", ColorSet(0b11100)),
    ("naya", ColorSet(0b11001)),
    // Tarkir wedges.
    ("abzan", ColorSet(0b10101)),
    ("jeskai", ColorSet(0b01011)),
    ("sultai", ColorSet(0b10110)),
    ("mardu", ColorSet(0b01101)),
    ("temur", ColorSet(0b11010)),
];

/// The five four-color nicknames.
const FOUR_COLOR: [(&str, ColorSet); 5] = [
    ("artifice", ColorSet(0b01111)),
    ("chaos", ColorSet(0b11110)),
    ("aggression", ColorSet(0b11101)),
    ("altruism", ColorSet(0b11011)),
    ("growth", ColorSet(0b10111)),
];

impl ColorSet {
    const MASK: u8 = 0b1_1111;

    /// No colors.
    pub const COLORLESS: Self = Self(0);
    /// White only.
    pub const WHITE: Self = Self(Color::White.bit());
    /// Blue only.
    pub const BLUE: Self = Self(Color::Blue.bit());
    /// Black only.
    pub const BLACK: Self = Self(Color::Black.bit());
    /// Red only.
    pub const RED: Self = Self(Color::Red.bit());
    /// Green only.
    pub const GREEN: Self = Self(Color::Green.bit());
    /// All five colors.
    pub const ALL: Self = Self(Self::MASK);

    /// A set holding exactly `color`.
    #[must_use]
    pub const fn of(color: Color) -> Self {
        Self(color.bit())
    }

    /// The raw bits (`0..=31`).
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Builds a set from raw bits, or `None` if any of the top three bits are set.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::MASK == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Builds a set from raw bits, ignoring the top three.
    #[must_use]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & Self::MASK)
    }

    /// This set plus `color`.
    #[must_use]
    pub const fn with(self, color: Color) -> Self {
        Self(self.0 | color.bit())
    }

    /// This set minus `color`.
    #[must_use]
    pub const fn without(self, color: Color) -> Self {
        Self(self.0 & !color.bit())
    }

    /// Whether `color` is in the set.
    #[must_use]
    pub const fn contains(self, color: Color) -> bool {
        self.0 & color.bit() != 0
    }

    /// Colors in either set.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Colors in both sets.
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Colors in `self` but not `other`.
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Colors in exactly one of the sets.
    #[must_use]
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// The colors *not* in this set.
    #[must_use]
    pub const fn complement(self) -> Self {
        Self(!self.0 & Self::MASK)
    }

    /// Whether this is the empty (colorless) set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The number of colors.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Exactly one color.
    #[must_use]
    pub const fn is_monocolor(self) -> bool {
        self.len() == 1
    }

    /// Two or more colors.
    #[must_use]
    pub const fn is_multicolor(self) -> bool {
        self.len() >= 2
    }

    /// Every color here is also in `other` (`self <= other`).
    #[must_use]
    pub const fn is_subset(self, other: Self) -> bool {
        self.0 & !other.0 == 0
    }

    /// Every color in `other` is also here (`self >= other`).
    #[must_use]
    pub const fn is_superset(self, other: Self) -> bool {
        other.is_subset(self)
    }

    /// The sets share at least one color.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// The sets share no colors.
    #[must_use]
    pub const fn is_disjoint(self, other: Self) -> bool {
        !self.intersects(other)
    }

    /// Compares by containment: `Less` if `self` is a proper subset, `Greater` if a proper
    /// superset, `Equal` if identical, `None` if neither contains the other.
    #[must_use]
    pub const fn subset_cmp(self, other: Self) -> Option<Ordering> {
        if self.0 == other.0 {
            Some(Ordering::Equal)
        } else if self.is_subset(other) {
            Some(Ordering::Less)
        } else if other.is_subset(self) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }

    /// Applies a Scryfall comparison, where `<=` means subset and `>=` means superset.
    ///
    /// `ColorSet::of(Red).compare(Cmp::Le, ColorSet::ALL)` is true: red fits in five colors.
    #[must_use]
    pub const fn compare(self, op: Cmp, other: Self) -> bool {
        op.holds(self.subset_cmp(other))
    }

    /// Iterates the colors in WUBRG order.
    #[must_use]
    pub const fn iter(self) -> Iter {
        Iter { bits: self.0 }
    }

    /// Iterates the colors in the alphabetical order Scryfall and MTGJSON list them in
    /// (`["B", "G", "U"]`), as opposed to [`iter`](Self::iter)'s WUBRG order.
    pub fn iter_alphabetical(self) -> impl Iterator<Item = Color> {
        Color::ALPHABETICAL
            .into_iter()
            .filter(move |color| self.contains(*color))
    }

    /// The nickname Scryfall uses for this set, if it has one: `azorius`, `esper`, `chaos`, or
    /// a single color name. `None` for colorless and for the five-color set.
    #[must_use]
    pub fn nickname(self) -> Option<&'static str> {
        NAMED
            .iter()
            .chain(FOUR_COLOR.iter())
            .find(|(_, set)| *set == self)
            .map(|(name, _)| *name)
    }

    /// Parses the color syntax of Scryfall's `c:` and `id:` keywords: letters in any order and
    /// case (`rg`, `WUBRG`), color names, and the guild, college, shard, wedge and four-color
    /// nicknames (`azorius`, `quandrix`, `bant`, `abzan`, `chaos`). `c` and `colorless` give
    /// the empty set.
    ///
    /// The `m`/`multicolor` and numeric forms are not sets; see [`ColorSpec`](crate::ColorSpec).
    ///
    /// # Errors
    ///
    /// [`ParseError::Empty`] for blank input, [`ParseError::UnknownColor`] for anything else
    /// that is not a color expression.
    pub fn from_scryfall(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        if s.eq_ignore_ascii_case("c") || s.eq_ignore_ascii_case("colorless") {
            return Ok(Self::COLORLESS);
        }
        if let Some((_, set)) = NAMED
            .iter()
            .chain(FOUR_COLOR.iter())
            .find(|(name, _)| s.eq_ignore_ascii_case(name))
        {
            return Ok(*set);
        }
        let mut set = Self::COLORLESS;
        for letter in s.chars() {
            match Color::from_letter(letter) {
                Some(color) => set = set.with(color),
                None => return Err(ParseError::UnknownColor(s.to_string())),
            }
        }
        Ok(set)
    }

    /// Appends the one-byte binary form (the raw bits).
    pub fn encode(self, out: &mut alloc::vec::Vec<u8>) {
        out.push(self.0);
    }

    /// Reads a set from the first byte of `bytes`, returning it and the bytes consumed (1).
    ///
    /// # Errors
    ///
    /// [`DecodeError::UnexpectedEof`] on empty input, [`DecodeError::InvalidEncoding`] if any of
    /// the top three bits are set.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let byte = *bytes.first().ok_or(DecodeError::UnexpectedEof)?;
        Self::from_bits(byte)
            .map(|set| (set, 1))
            .ok_or(DecodeError::InvalidEncoding)
    }
}

impl fmt::Debug for ColorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColorSet({self})")
    }
}

/// Letters in WUBRG order, or `C` for the empty set. Round-trips through
/// [`ColorSet::from_scryfall`].
impl fmt::Display for ColorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("C");
        }
        for color in self.iter() {
            fmt::Write::write_char(f, color.letter())?;
        }
        Ok(())
    }
}

impl FromStr for ColorSet {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_scryfall(s)
    }
}

impl From<Color> for ColorSet {
    fn from(color: Color) -> Self {
        Self::of(color)
    }
}

impl FromIterator<Color> for ColorSet {
    fn from_iter<I: IntoIterator<Item = Color>>(iter: I) -> Self {
        iter.into_iter().fold(Self::COLORLESS, Self::with)
    }
}

impl Extend<Color> for ColorSet {
    fn extend<I: IntoIterator<Item = Color>>(&mut self, iter: I) {
        for color in iter {
            *self = self.with(color);
        }
    }
}

impl IntoIterator for ColorSet {
    type Item = Color;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

impl BitOr for ColorSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitAnd for ColorSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl BitXor for ColorSet {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(rhs)
    }
}

impl Sub for ColorSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}

impl Not for ColorSet {
    type Output = Self;

    fn not(self) -> Self {
        self.complement()
    }
}

/// Iterator over a [`ColorSet`] in WUBRG order.
#[derive(Clone, Debug)]
pub struct Iter {
    bits: u8,
}

impl Iterator for Iter {
    type Item = Color;

    fn next(&mut self) -> Option<Color> {
        if self.bits == 0 {
            return None;
        }
        let index = self.bits.trailing_zeros() as u8;
        self.bits &= self.bits - 1;
        Color::from_index(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.bits.count_ones() as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Color> {
        if self.bits == 0 {
            return None;
        }
        let index = 7 - self.bits.leading_zeros() as u8;
        self.bits &= !(1 << index);
        Color::from_index(index)
    }
}

impl ExactSizeIterator for Iter {}
impl FusedIterator for Iter {}
