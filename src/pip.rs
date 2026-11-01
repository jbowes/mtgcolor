//! Mana symbols: the individual pips that make up a mana cost.

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use crate::color::{Color, ColorSet};
use crate::error::ParseError;

/// The ten pairs of colors, in the one direction Scryfall prints each: `{W/U}`, never `{U/W}`.
const PAIRS: [(Color, Color); 10] = [
    (Color::White, Color::Blue),
    (Color::White, Color::Black),
    (Color::Black, Color::Red),
    (Color::Black, Color::Green),
    (Color::Blue, Color::Black),
    (Color::Blue, Color::Red),
    (Color::Red, Color::Green),
    (Color::Red, Color::White),
    (Color::Green, Color::White),
    (Color::Green, Color::Blue),
];

/// Two different colors, held in Scryfall's canonical print direction.
///
/// `{W/U}` and `{U/W}` are the same symbol; building a pair from either order yields the same
/// value, so equality and hashing ignore the order the text happened to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorPair(u8);

impl ColorPair {
    /// The pair `{a/b}` in canonical direction, or `None` if `a == b`.
    #[must_use]
    pub const fn new(a: Color, b: Color) -> Option<Self> {
        let mut i = 0;
        while i < PAIRS.len() {
            let (x, y) = PAIRS[i];
            if (x as u8 == a as u8 && y as u8 == b as u8)
                || (x as u8 == b as u8 && y as u8 == a as u8)
            {
                return Some(Self(i as u8));
            }
            i += 1;
        }
        None
    }

    /// The pair at position `index` of the canonical list, `0..10`.
    #[must_use]
    pub const fn from_index(index: u8) -> Option<Self> {
        if (index as usize) < PAIRS.len() {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Position in the canonical list, `0..10`.
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// The color printed first.
    #[must_use]
    pub const fn first(self) -> Color {
        PAIRS[self.0 as usize].0
    }

    /// The color printed second.
    #[must_use]
    pub const fn second(self) -> Color {
        PAIRS[self.0 as usize].1
    }

    /// Both colors as a set.
    #[must_use]
    pub const fn colors(self) -> ColorSet {
        ColorSet::of(self.first()).with(self.second())
    }
}

/// A single mana symbol as it appears in a mana cost.
///
/// The variants are the symbols Scryfall documents; anything else fails to parse rather than
/// being guessed at. Parsing is case-insensitive and printing is canonical, so
/// `Pip::parse("{u/w}")` prints as `{W/U}`.
///
/// Matching on this enum must include a wildcard arm: new symbols are added as they are printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Pip {
    /// Generic mana: `{0}`, `{2}`, `{16}`, `{1000000}`.
    Generic(u32),
    /// `{X}`
    X,
    /// `{Y}`
    Y,
    /// `{Z}`
    Z,
    /// One colored mana: `{W}`.
    Colored(Color),
    /// `{C}`, colorless mana.
    Colorless,
    /// `{S}`, snow mana.
    Snow,
    /// Hybrid of two colors: `{W/U}`.
    Hybrid(ColorPair),
    /// Monocolored hybrid, two generic or one colored: `{2/W}`.
    Monohybrid(Color),
    /// Colorless-or-colored hybrid: `{C/W}`.
    ColorlessHybrid(Color),
    /// A color or 2 life: `{W/P}`.
    Phyrexian(Color),
    /// Colorless or 2 life: `{C/P}`.
    PhyrexianColorless,
    /// Either of two colors or 2 life: `{G/U/P}`.
    PhyrexianHybrid(ColorPair),
    /// Any one color or 2 life: `{H}`.
    PhyrexianAny,
    /// Half a colored mana: `{HW}`, `{HR}`.
    Half(Color),
    /// Half a generic mana: `{½}`.
    HalfGeneric,
    /// Infinite generic mana: `{∞}`.
    Infinity,
    /// One mana from a legendary source: `{L}`.
    Legendary,
    /// A potential land drop: `{D}`. Appears in costs but is not mana.
    LandDrop,
}

impl Pip {
    /// `{a/b}`, normalised, or `None` if `a == b`.
    #[must_use]
    pub const fn hybrid(a: Color, b: Color) -> Option<Self> {
        match ColorPair::new(a, b) {
            Some(pair) => Some(Self::Hybrid(pair)),
            None => None,
        }
    }

    /// `{a/b/P}`, normalised, or `None` if `a == b`.
    #[must_use]
    pub const fn phyrexian_hybrid(a: Color, b: Color) -> Option<Self> {
        match ColorPair::new(a, b) {
            Some(pair) => Some(Self::PhyrexianHybrid(pair)),
            None => None,
        }
    }

    /// The colors this symbol contributes to a card's color and color identity.
    ///
    /// Hybrids give both colors; Phyrexian, monohybrid and half symbols give their one color;
    /// generic, colorless, snow and the rest give none.
    #[must_use]
    pub const fn colors(self) -> ColorSet {
        match self {
            Self::Colored(c)
            | Self::Monohybrid(c)
            | Self::ColorlessHybrid(c)
            | Self::Phyrexian(c)
            | Self::Half(c) => ColorSet::of(c),
            Self::Hybrid(pair) | Self::PhyrexianHybrid(pair) => pair.colors(),
            Self::Generic(_)
            | Self::X
            | Self::Y
            | Self::Z
            | Self::Colorless
            | Self::Snow
            | Self::PhyrexianColorless
            | Self::PhyrexianAny
            | Self::HalfGeneric
            | Self::Infinity
            | Self::Legendary
            | Self::LandDrop => ColorSet::COLORLESS,
        }
    }

    /// This symbol's contribution to mana value, in halves (`{HW}` is 1, `{W}` is 2). `None`
    /// for `{∞}`, which has no finite value.
    ///
    /// Hybrids count as their largest half, so `{2/W}` is 2 and `{W/U}` is 1; variables and
    /// `{D}` are 0.
    #[must_use]
    pub const fn mana_value_halves(self) -> Option<u64> {
        match self {
            Self::Generic(n) => Some(n as u64 * 2),
            Self::X | Self::Y | Self::Z | Self::LandDrop => Some(0),
            Self::Half(_) | Self::HalfGeneric => Some(1),
            Self::Monohybrid(_) => Some(4),
            Self::Colored(_)
            | Self::Colorless
            | Self::Snow
            | Self::Hybrid(_)
            | Self::ColorlessHybrid(_)
            | Self::Phyrexian(_)
            | Self::PhyrexianColorless
            | Self::PhyrexianHybrid(_)
            | Self::PhyrexianAny
            | Self::Legendary => Some(2),
            Self::Infinity => None,
        }
    }

    /// Contribution to mana value as a number; infinite for `{∞}`.
    #[must_use]
    pub fn mana_value(self) -> f64 {
        self.mana_value_halves()
            .map_or(f64::INFINITY, |h| h as f64 / 2.0)
    }

    /// A hybrid symbol of any kind (Scryfall's `is:hybrid`): two colors, a monohybrid, or a
    /// colorless hybrid.
    #[must_use]
    pub const fn is_hybrid(self) -> bool {
        matches!(
            self,
            Self::Hybrid(_)
                | Self::Monohybrid(_)
                | Self::ColorlessHybrid(_)
                | Self::PhyrexianHybrid(_)
        )
    }

    /// Payable with 2 life (Scryfall's `is:phyrexian`).
    #[must_use]
    pub const fn is_phyrexian(self) -> bool {
        matches!(
            self,
            Self::Phyrexian(_)
                | Self::PhyrexianColorless
                | Self::PhyrexianHybrid(_)
                | Self::PhyrexianAny
        )
    }

    /// `{X}`, `{Y}` or `{Z}`.
    #[must_use]
    pub const fn is_variable(self) -> bool {
        matches!(self, Self::X | Self::Y | Self::Z)
    }

    /// Parses one symbol written with braces, such as `{2/W}`.
    ///
    /// # Errors
    ///
    /// [`ParseError::MalformedBraces`] if the braces are missing or nested,
    /// [`ParseError::UnknownSymbol`] if the symbol is not a mana symbol Scryfall documents.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let body = s
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .filter(|body| !body.contains(['{', '}']))
            .ok_or(ParseError::MalformedBraces)?;
        Self::parse_body(body)
    }

    /// Parses the text between the braces: `W/U`, `2`, `∞`.
    pub(crate) fn parse_body(body: &str) -> Result<Self, ParseError> {
        let unknown = || ParseError::UnknownSymbol(body.to_string());
        if body.is_empty() {
            return Err(unknown());
        }
        if body.bytes().all(|b| b.is_ascii_digit()) {
            if body.len() > 1 && body.starts_with('0') {
                return Err(unknown());
            }
            return body.parse().map(Self::Generic).map_err(|_| unknown());
        }
        let mut parts = body.split('/');
        let a = parts.next().unwrap_or_default();
        let b = parts.next();
        let c = parts.next();
        if parts.next().is_some() {
            return Err(unknown());
        }
        match (a, b, c) {
            (a, None, None) => Self::parse_single(a).ok_or_else(unknown),
            (a, Some(b), None) => Self::parse_pair(a, b).ok_or_else(unknown),
            (a, Some(b), Some(c)) if c.eq_ignore_ascii_case("p") => {
                let (a, b) = (
                    one_color(a).ok_or_else(unknown)?,
                    one_color(b).ok_or_else(unknown)?,
                );
                Self::phyrexian_hybrid(a, b).ok_or_else(unknown)
            }
            _ => Err(unknown()),
        }
    }

    fn parse_single(a: &str) -> Option<Self> {
        match a {
            "∞" => return Some(Self::Infinity),
            "½" => return Some(Self::HalfGeneric),
            _ => {}
        }
        let mut chars = a.chars();
        let first = chars.next()?.to_ascii_uppercase();
        match (first, chars.next()) {
            (c, None) => match c {
                'X' => Some(Self::X),
                'Y' => Some(Self::Y),
                'Z' => Some(Self::Z),
                'C' => Some(Self::Colorless),
                'S' => Some(Self::Snow),
                'L' => Some(Self::Legendary),
                'D' => Some(Self::LandDrop),
                'H' => Some(Self::PhyrexianAny),
                c => Color::from_letter(c).map(Self::Colored),
            },
            ('H', Some(c)) if chars.next().is_none() => Color::from_letter(c).map(Self::Half),
            _ => None,
        }
    }

    fn parse_pair(a: &str, b: &str) -> Option<Self> {
        if b.eq_ignore_ascii_case("p") {
            return if a.eq_ignore_ascii_case("c") {
                Some(Self::PhyrexianColorless)
            } else {
                one_color(a).map(Self::Phyrexian)
            };
        }
        let right = one_color(b)?;
        if a == "2" {
            Some(Self::Monohybrid(right))
        } else if a.eq_ignore_ascii_case("c") {
            Some(Self::ColorlessHybrid(right))
        } else {
            Self::hybrid(one_color(a)?, right)
        }
    }

    /// Writes the symbol without braces, as it appears between them.
    fn write_body(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(n) => write!(f, "{n}"),
            Self::X => f.write_str("X"),
            Self::Y => f.write_str("Y"),
            Self::Z => f.write_str("Z"),
            Self::Colored(c) => write!(f, "{c}"),
            Self::Colorless => f.write_str("C"),
            Self::Snow => f.write_str("S"),
            Self::Hybrid(p) => write!(f, "{}/{}", p.first(), p.second()),
            Self::Monohybrid(c) => write!(f, "2/{c}"),
            Self::ColorlessHybrid(c) => write!(f, "C/{c}"),
            Self::Phyrexian(c) => write!(f, "{c}/P"),
            Self::PhyrexianColorless => f.write_str("C/P"),
            Self::PhyrexianHybrid(p) => write!(f, "{}/{}/P", p.first(), p.second()),
            Self::PhyrexianAny => f.write_str("H"),
            Self::Half(c) => write!(f, "H{c}"),
            Self::HalfGeneric => f.write_str("½"),
            Self::Infinity => f.write_str("∞"),
            Self::Legendary => f.write_str("L"),
            Self::LandDrop => f.write_str("D"),
        }
    }
}

/// A single color letter, in either case.
fn one_color(s: &str) -> Option<Color> {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Color::from_letter(c),
        _ => None,
    }
}

impl fmt::Display for Pip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        self.write_body(f)?;
        f.write_str("}")
    }
}

impl FromStr for Pip {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Color> for Pip {
    fn from(color: Color) -> Self {
        Self::Colored(color)
    }
}

/// Renders a pip to a `String`; handy for keys and tests.
impl From<Pip> for String {
    fn from(pip: Pip) -> Self {
        pip.to_string()
    }
}
