//! Mana costs: ordered lists of pips.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::binary::{read_varint, write_varint};
use crate::cmp::Cmp;
use crate::color::{Color, ColorSet};
use crate::error::{DecodeError, ParseError};
use crate::pip::Pip;

/// A mana cost such as `{2}{W}{W}`: the pips in the order they are printed.
///
/// An empty cost is a valid value and prints as the empty string; it is what lands have. It is
/// not the same as `{0}`, which is a cost of one zero-generic pip. Callers that need to tell "no
/// cost" from "no mana cost field" should use `Option<ManaCost>`.
///
/// Equality is order-sensitive, like the printed cost. Use [`canonical`](Self::canonical) or
/// [`compare`](Self::compare) to ignore order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ManaCost {
    pips: Vec<Pip>,
}

impl ManaCost {
    /// An empty cost.
    #[must_use]
    pub const fn new() -> Self {
        Self { pips: Vec::new() }
    }

    /// The pips in printed order.
    #[must_use]
    pub fn pips(&self) -> &[Pip] {
        &self.pips
    }

    /// The number of pips.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pips.len()
    }

    /// Whether there are no pips.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pips.is_empty()
    }

    /// Appends a pip.
    pub fn push(&mut self, pip: Pip) {
        self.pips.push(pip);
    }

    /// Iterates the pips in printed order.
    pub fn iter(&self) -> core::slice::Iter<'_, Pip> {
        self.pips.iter()
    }

    /// Parses a mana cost as Scryfall and MTGJSON print it: zero or more `{...}` symbols with
    /// nothing between them. The empty string is the empty cost.
    ///
    /// # Errors
    ///
    /// [`ParseError::MalformedBraces`], [`ParseError::UnexpectedText`] for anything outside
    /// braces, or [`ParseError::UnknownSymbol`] for a symbol that is not a documented one.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let mut pips = Vec::new();
        let mut rest = s;
        while !rest.is_empty() {
            if rest.starts_with('}') {
                return Err(ParseError::MalformedBraces);
            }
            if !rest.starts_with('{') {
                let text = rest.split('{').next().unwrap_or(rest);
                return Err(ParseError::UnexpectedText(text.to_string()));
            }
            let end = rest.find('}').ok_or(ParseError::MalformedBraces)?;
            pips.push(Pip::parse(&rest[..=end])?);
            rest = &rest[end + 1..];
        }
        Ok(Self { pips })
    }

    /// Parses a mana cost the way Scryfall's `m:` keyword reads one: braces are optional around
    /// symbols that are not split, whitespace between symbols is ignored, and case does not
    /// matter. `2WW`, `{2}{W}{W}` and `2 w w` are the same cost.
    ///
    /// Split symbols (`{2/W}`, `{W/P}`, `{HW}`) must keep their braces, as on Scryfall. A bare
    /// `H` is [`Pip::PhyrexianAny`].
    ///
    /// # Errors
    ///
    /// As [`parse`](Self::parse).
    pub fn parse_query(s: &str) -> Result<Self, ParseError> {
        let mut pips = Vec::new();
        let mut rest = s.trim_start();
        while let Some(first) = rest.chars().next() {
            let len = match first {
                '{' => rest.find('}').ok_or(ParseError::MalformedBraces)? + 1,
                '}' => return Err(ParseError::MalformedBraces),
                c if c.is_ascii_digit() => rest
                    .find(|c: char| !c.is_ascii_digit())
                    .unwrap_or(rest.len()),
                c => c.len_utf8(),
            };
            let symbol = &rest[..len];
            pips.push(if symbol.starts_with('{') {
                Pip::parse(symbol)?
            } else {
                Pip::parse_body(symbol)?
            });
            rest = rest[len..].trim_start();
        }
        Ok(Self { pips })
    }

    /// The union of the colors of every pip: the card's color as far as its cost goes.
    #[must_use]
    pub fn colors(&self) -> ColorSet {
        self.pips
            .iter()
            .fold(ColorSet::COLORLESS, |set, pip| set | pip.colors())
    }

    /// Mana value in halves, or `None` if the cost contains `{∞}`. `{HW}` is 1 and `{W}` is 2.
    #[must_use]
    pub fn mana_value_halves(&self) -> Option<u64> {
        self.pips.iter().try_fold(0u64, |sum, pip| {
            pip.mana_value_halves()
                .map(|halves| sum.saturating_add(halves))
        })
    }

    /// Mana value, with `{X}` as 0 and `{∞}` as infinity.
    #[must_use]
    pub fn mana_value(&self) -> f64 {
        self.mana_value_halves()
            .map_or(f64::INFINITY, |h| h as f64 / 2.0)
    }

    /// Devotion to `color`: how many pips contain it. A hybrid pip counts toward both colors.
    #[must_use]
    pub fn devotion(&self, color: Color) -> usize {
        self.pips
            .iter()
            .filter(|pip| pip.colors().contains(color))
            .count()
    }

    /// How many `{X}` pips the cost has (`{Y}` and `{Z}` are not counted).
    #[must_use]
    pub fn x_count(&self) -> usize {
        self.pips.iter().filter(|pip| matches!(pip, Pip::X)).count()
    }

    /// Whether any pip is hybrid (Scryfall's `is:hybrid`).
    #[must_use]
    pub fn has_hybrid(&self) -> bool {
        self.pips.iter().any(|pip| pip.is_hybrid())
    }

    /// Whether any pip is payable with life (Scryfall's `is:phyrexian`).
    #[must_use]
    pub fn has_phyrexian(&self) -> bool {
        self.pips.iter().any(|pip| pip.is_phyrexian())
    }

    /// The same pips in a fixed sort order, so costs that differ only in order become equal.
    #[must_use]
    pub fn canonical(&self) -> Self {
        let mut pips = self.pips.clone();
        pips.sort_unstable();
        Self { pips }
    }

    /// Compares the two costs as bags of symbols, ignoring order: `Less` if `self` has a subset
    /// of `other`'s symbols (counting repeats), `Greater` if a superset, `Equal` if the same,
    /// `None` if neither contains the other.
    #[must_use]
    pub fn subset_cmp(&self, other: &Self) -> Option<Ordering> {
        let (a, b) = (self.canonical(), other.canonical());
        let (a_in_b, b_in_a) = (
            bag_contains(&b.pips, &a.pips),
            bag_contains(&a.pips, &b.pips),
        );
        match (a_in_b, b_in_a) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }

    /// Applies a Scryfall `m` comparison: `>=` means `self` includes every symbol of `other`
    /// and possibly more.
    #[must_use]
    pub fn compare(&self, op: Cmp, other: &Self) -> bool {
        op.holds(self.subset_cmp(other))
    }

    /// Whether `self` includes every symbol of `other`, counting repeats (`m:` with its default
    /// operator).
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        self.compare(Cmp::Ge, other)
    }

    /// Appends the binary form: the pip count as a varint, then each pip.
    pub fn encode(&self, out: &mut Vec<u8>) {
        write_varint(out, self.pips.len() as u64);
        for pip in &self.pips {
            pip.encode(out);
        }
    }

    /// Reads a cost from the front of `bytes`, returning it and the bytes consumed.
    ///
    /// # Errors
    ///
    /// See [`Pip::decode`]; also [`DecodeError::UnexpectedEof`] if the declared count exceeds
    /// the bytes available.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (count, mut used) = read_varint(bytes)?;
        // Every pip takes at least one byte, so a count beyond the input is truncated input.
        // Checking first keeps a hostile count from reserving memory.
        let count = usize::try_from(count).map_err(|_| DecodeError::UnexpectedEof)?;
        if count > bytes.len() - used {
            return Err(DecodeError::UnexpectedEof);
        }
        let mut pips = Vec::with_capacity(count);
        for _ in 0..count {
            let (pip, n) = Pip::decode(&bytes[used..])?;
            pips.push(pip);
            used += n;
        }
        Ok((Self { pips }, used))
    }

    /// The binary form as a new vector.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pips.len() + 1);
        self.encode(&mut out);
        out
    }

    /// Reads a cost that must span all of `bytes`.
    ///
    /// # Errors
    ///
    /// As [`decode`](Self::decode), plus [`DecodeError::TrailingBytes`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (cost, used) = Self::decode(bytes)?;
        if used == bytes.len() {
            Ok(cost)
        } else {
            Err(DecodeError::TrailingBytes)
        }
    }
}

/// Whether the sorted slice `haystack` contains every element of the sorted slice `needles`,
/// counting repeats.
fn bag_contains(haystack: &[Pip], needles: &[Pip]) -> bool {
    let mut remaining = haystack.iter();
    needles
        .iter()
        .all(|needle| remaining.any(|have| have == needle))
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pips.iter().try_for_each(|pip| write!(f, "{pip}"))
    }
}

impl FromStr for ManaCost {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Vec<Pip>> for ManaCost {
    fn from(pips: Vec<Pip>) -> Self {
        Self { pips }
    }
}

impl FromIterator<Pip> for ManaCost {
    fn from_iter<I: IntoIterator<Item = Pip>>(iter: I) -> Self {
        Self {
            pips: iter.into_iter().collect(),
        }
    }
}

impl Extend<Pip> for ManaCost {
    fn extend<I: IntoIterator<Item = Pip>>(&mut self, iter: I) {
        self.pips.extend(iter);
    }
}

impl<'a> IntoIterator for &'a ManaCost {
    type Item = &'a Pip;
    type IntoIter = core::slice::Iter<'a, Pip>;

    fn into_iter(self) -> Self::IntoIter {
        self.pips.iter()
    }
}

impl IntoIterator for ManaCost {
    type Item = Pip;
    type IntoIter = alloc::vec::IntoIter<Pip>;

    fn into_iter(self) -> Self::IntoIter {
        self.pips.into_iter()
    }
}

/// Renders a cost to a `String`.
impl From<&ManaCost> for String {
    fn from(cost: &ManaCost) -> Self {
        cost.to_string()
    }
}
