//! Every `{...}` symbol that can appear in Oracle text, and a scanner for finding them.

use alloc::string::ToString;
use core::fmt;
use core::str::FromStr;

use crate::error::ParseError;
use crate::pip::Pip;

/// A symbol as written in Oracle text or a mana cost: mana, or one of the other symbols
/// Scryfall documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Symbol {
    /// Any mana symbol that can be paid or produced.
    Mana(Pip),
    /// `{T}`, tap this permanent.
    Tap,
    /// `{Q}`, untap this permanent.
    Untap,
    /// `{E}`, an energy counter.
    Energy,
    /// `{P}`, a modal budget pawprint. This is *not* Phyrexian mana.
    Pawprint,
    /// `{PW}`, the planeswalker symbol.
    Planeswalker,
    /// `{CHAOS}`, the chaos symbol.
    Chaos,
    /// `{A}`, an acorn counter.
    Acorn,
    /// `{TK}`, a ticket counter.
    Ticket,
}

impl Symbol {
    /// Parses one symbol written with braces.
    ///
    /// # Errors
    ///
    /// [`ParseError::MalformedBraces`] or [`ParseError::UnknownSymbol`].
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let body = s
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .filter(|body| !body.contains(['{', '}']))
            .ok_or(ParseError::MalformedBraces)?;
        Self::parse_body(body)
    }

    fn parse_body(body: &str) -> Result<Self, ParseError> {
        const OTHERS: [(&str, Symbol); 8] = [
            ("T", Symbol::Tap),
            ("Q", Symbol::Untap),
            ("E", Symbol::Energy),
            ("P", Symbol::Pawprint),
            ("PW", Symbol::Planeswalker),
            ("CHAOS", Symbol::Chaos),
            ("A", Symbol::Acorn),
            ("TK", Symbol::Ticket),
        ];
        match OTHERS
            .iter()
            .find(|(name, _)| body.eq_ignore_ascii_case(name))
        {
            Some((_, symbol)) => Ok(*symbol),
            None => Pip::parse_body(body).map(Self::Mana),
        }
    }

    /// The mana pip, if this is a mana symbol.
    #[must_use]
    pub const fn pip(self) -> Option<Pip> {
        match self {
            Self::Mana(pip) => Some(pip),
            _ => None,
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mana(pip) => write!(f, "{pip}"),
            Self::Tap => f.write_str("{T}"),
            Self::Untap => f.write_str("{Q}"),
            Self::Energy => f.write_str("{E}"),
            Self::Pawprint => f.write_str("{P}"),
            Self::Planeswalker => f.write_str("{PW}"),
            Self::Chaos => f.write_str("{CHAOS}"),
            Self::Acorn => f.write_str("{A}"),
            Self::Ticket => f.write_str("{TK}"),
        }
    }
}

impl FromStr for Symbol {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Pip> for Symbol {
    fn from(pip: Pip) -> Self {
        Self::Mana(pip)
    }
}

/// Iterator over the `{...}` symbols in a piece of text, in order.
///
/// Each item is the parse result of one braced token, so a symbol this crate does not know shows
/// up as an `Err` instead of being skipped. Braces that do not pair up are ignored.
#[derive(Clone, Debug)]
pub struct Symbols<'a> {
    rest: &'a str,
    skip_reminder: bool,
}

impl<'a> Symbols<'a> {
    /// Scans all of `text`, including parenthesised reminder text.
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self {
            rest: text,
            skip_reminder: false,
        }
    }

    /// Scans `text` but skips anything inside parentheses, which is reminder text. Reminder text
    /// does not count toward a card's color identity.
    #[must_use]
    pub const fn outside_reminder_text(text: &'a str) -> Self {
        Self {
            rest: text,
            skip_reminder: true,
        }
    }
}

impl Iterator for Symbols<'_> {
    type Item = Result<Symbol, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let at = self
                .rest
                .find(|c| c == '{' || (self.skip_reminder && c == '('))?;
            if self.rest[at..].starts_with('(') {
                // Skip to the matching close, or to the end if it never comes.
                let mut depth = 0usize;
                let end = self.rest[at..]
                    .char_indices()
                    .find_map(|(i, c)| {
                        match c {
                            '(' => depth += 1,
                            ')' => depth -= 1,
                            _ => {}
                        }
                        (depth == 0).then_some(at + i + 1)
                    })
                    .unwrap_or(self.rest.len());
                self.rest = &self.rest[end..];
                continue;
            }
            let after = &self.rest[at + 1..];
            let Some(close) = after.find('}') else {
                self.rest = "";
                return None;
            };
            let body = &after[..close];
            if let Some(inner) = body.rfind('{') {
                // A stray `{` before the real one; restart from it.
                self.rest = &after[inner..];
                continue;
            }
            self.rest = &after[close + 1..];
            return Some(Symbol::parse_body(body).map_err(|e| match e {
                ParseError::UnknownSymbol(_) => ParseError::UnknownSymbol(body.to_string()),
                other => other,
            }));
        }
    }
}
