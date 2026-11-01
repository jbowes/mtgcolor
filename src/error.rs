//! Error types for parsing and binary decoding.

use alloc::string::String;
use core::fmt;

/// A failure to parse colors, pips, mana costs or symbols from text.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The input was empty or only whitespace where a value was required.
    Empty,
    /// A `{` with no matching `}`, a stray `}`, or nested braces.
    MalformedBraces,
    /// Text outside of braces in a context that only allows brace-delimited symbols.
    UnexpectedText(String),
    /// A `{...}` symbol that is not a known mana symbol.
    UnknownSymbol(String),
    /// A color letter, color name or nickname that is not recognised.
    UnknownColor(String),
    /// A color count outside `0..=5`.
    InvalidColorCount(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::MalformedBraces => f.write_str("unbalanced or nested braces"),
            Self::UnexpectedText(s) => write!(f, "unexpected text outside braces: {s:?}"),
            Self::UnknownSymbol(s) => write!(f, "unknown symbol: {s:?}"),
            Self::UnknownColor(s) => write!(f, "unknown color: {s:?}"),
            Self::InvalidColorCount(s) => write!(f, "invalid color count: {s:?}"),
        }
    }
}

impl core::error::Error for ParseError {}

/// A failure to decode the compact binary form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// The input ended before the value was complete.
    UnexpectedEof,
    /// A tag or payload that no encoder produces.
    InvalidEncoding,
    /// A valid value written in a longer form than the canonical one.
    NonCanonical,
    /// Bytes were left over after a complete value (`from_bytes` only).
    TrailingBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnexpectedEof => "unexpected end of input",
            Self::InvalidEncoding => "invalid encoding",
            Self::NonCanonical => "non-canonical encoding",
            Self::TrailingBytes => "trailing bytes after value",
        })
    }
}

impl core::error::Error for DecodeError {}
