//! Error types for parsing and binary decoding.

use alloc::string::String;
use core::fmt;

/// A failure to parse colors from text.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The input was empty or only whitespace where a value was required.
    Empty,
    /// A color letter, color name or nickname that is not recognised.
    UnknownColor(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty input"),
            Self::UnknownColor(s) => write!(f, "unknown color: {s:?}"),
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
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnexpectedEof => "unexpected end of input",
            Self::InvalidEncoding => "invalid encoding",
        })
    }
}

impl core::error::Error for DecodeError {}
