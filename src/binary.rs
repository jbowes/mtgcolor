//! The compact binary encoding.
//!
//! Every value has exactly one valid encoding, so encoded bytes can be compared, hashed and used
//! as database keys. Decoders reject anything an encoder would not produce.
//!
//! | Value | Encoding |
//! | --- | --- |
//! | [`ColorSet`], [`ColorIdentity`](crate::ColorIdentity) | 1 byte: bit 0 W, 1 U, 2 B, 3 R, 4 G; bits 5-7 zero |
//! | [`Pip`] | 1 byte: high nibble is the kind, low nibble the payload (below) |
//! | `Pip::Generic(n)`, `n >= 15` | `0x0F`, then `n` as a LEB128 varint |
//! | [`ManaCost`](crate::ManaCost) | the pip count as a LEB128 varint, then each pip |
//!
//! Pip kinds (high nibble), payload (low nibble):
//!
//! | Kind | Pip | Payload |
//! | --- | --- | --- |
//! | `0x0` | `Generic(n)` | `n` for `0..=14`, `0xF` means a varint follows |
//! | `0x1` | `Colored` | color index (WUBRG, `0..=4`) |
//! | `0x2` | `Hybrid` | [`ColorPair`] index `0..=9` |
//! | `0x3` | `Monohybrid` | color index |
//! | `0x4` | `ColorlessHybrid` | color index |
//! | `0x5` | `Phyrexian` | color index |
//! | `0x6` | `PhyrexianHybrid` | pair index |
//! | `0x7` | `Half` | color index |
//! | `0x8` | `X`, `Y`, `Z` | `0`, `1`, `2` |
//! | `0x9` | `Colorless`, `Snow`, `Infinity`, `HalfGeneric`, `Legendary`, `LandDrop`, `PhyrexianColorless`, `PhyrexianAny` | `0..=7` in that order |
//!
//! Kinds `0xA` to `0xF` are reserved.
//!
//! So `{2}{W}{W}` is the four bytes `03 02 10 10` where its text form is eight.

use alloc::vec::Vec;

use crate::color::{Color, ColorSet};
use crate::error::DecodeError;
use crate::pip::{ColorPair, Pip};

/// Pips with `Generic(n)` below this value use the one-byte form.
const SMALL_GENERIC_LIMIT: u8 = 15;
const GENERIC_ESCAPE: u8 = 0x0F;

pub(crate) fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value & 0x7F) as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

/// Reads a canonical LEB128 value of at most 64 bits.
pub(crate) fn read_varint(bytes: &[u8]) -> Result<(u64, usize), DecodeError> {
    let mut value: u64 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        let shift = 7 * i as u32;
        let chunk = u64::from(byte & 0x7F);
        if shift >= 64 || (shift == 63 && chunk > 1) {
            return Err(DecodeError::InvalidEncoding);
        }
        value |= chunk << shift;
        if byte & 0x80 == 0 {
            if i > 0 && byte == 0 {
                return Err(DecodeError::NonCanonical);
            }
            return Ok((value, i + 1));
        }
    }
    Err(DecodeError::UnexpectedEof)
}

impl Pip {
    /// Appends the pip's binary form: one byte, or `0x0F` plus a varint for generic mana of 15
    /// or more.
    pub fn encode(self, out: &mut Vec<u8>) {
        let color = |kind: u8, c: Color| kind << 4 | c.index();
        let byte = match self {
            Self::Generic(n) => match u8::try_from(n) {
                Ok(small) if small < SMALL_GENERIC_LIMIT => small,
                _ => {
                    out.push(GENERIC_ESCAPE);
                    write_varint(out, u64::from(n));
                    return;
                }
            },
            Self::Colored(c) => color(0x1, c),
            Self::Hybrid(p) => 0x20 | p.index(),
            Self::Monohybrid(c) => color(0x3, c),
            Self::ColorlessHybrid(c) => color(0x4, c),
            Self::Phyrexian(c) => color(0x5, c),
            Self::PhyrexianHybrid(p) => 0x60 | p.index(),
            Self::Half(c) => color(0x7, c),
            Self::X => 0x80,
            Self::Y => 0x81,
            Self::Z => 0x82,
            Self::Colorless => 0x90,
            Self::Snow => 0x91,
            Self::Infinity => 0x92,
            Self::HalfGeneric => 0x93,
            Self::Legendary => 0x94,
            Self::LandDrop => 0x95,
            Self::PhyrexianColorless => 0x96,
            Self::PhyrexianAny => 0x97,
        };
        out.push(byte);
    }

    /// Reads one pip from the front of `bytes`, returning it and the bytes consumed.
    ///
    /// # Errors
    ///
    /// [`DecodeError::UnexpectedEof`] if `bytes` is too short, [`DecodeError::InvalidEncoding`]
    /// for a reserved or out-of-range tag, [`DecodeError::NonCanonical`] for a generic amount
    /// below 15 written in the long form or a padded varint.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let first = *bytes.first().ok_or(DecodeError::UnexpectedEof)?;
        let (kind, payload) = (first >> 4, first & 0x0F);
        let color = || Color::from_index(payload).ok_or(DecodeError::InvalidEncoding);
        let pair = || ColorPair::from_index(payload).ok_or(DecodeError::InvalidEncoding);
        let pip = match kind {
            0x0 if payload == GENERIC_ESCAPE => {
                let (value, used) = read_varint(&bytes[1..])?;
                let n = u32::try_from(value).map_err(|_| DecodeError::InvalidEncoding)?;
                if n < u32::from(SMALL_GENERIC_LIMIT) {
                    return Err(DecodeError::NonCanonical);
                }
                return Ok((Self::Generic(n), 1 + used));
            }
            0x0 => Self::Generic(u32::from(payload)),
            0x1 => Self::Colored(color()?),
            0x2 => Self::Hybrid(pair()?),
            0x3 => Self::Monohybrid(color()?),
            0x4 => Self::ColorlessHybrid(color()?),
            0x5 => Self::Phyrexian(color()?),
            0x6 => Self::PhyrexianHybrid(pair()?),
            0x7 => Self::Half(color()?),
            0x8 => match payload {
                0 => Self::X,
                1 => Self::Y,
                2 => Self::Z,
                _ => return Err(DecodeError::InvalidEncoding),
            },
            0x9 => match payload {
                0 => Self::Colorless,
                1 => Self::Snow,
                2 => Self::Infinity,
                3 => Self::HalfGeneric,
                4 => Self::Legendary,
                5 => Self::LandDrop,
                6 => Self::PhyrexianColorless,
                7 => Self::PhyrexianAny,
                _ => return Err(DecodeError::InvalidEncoding),
            },
            _ => return Err(DecodeError::InvalidEncoding),
        };
        Ok((pip, 1))
    }
}

impl ColorSet {
    /// The one-byte binary form.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        self.bits()
    }

    /// Reads the one-byte binary form.
    ///
    /// # Errors
    ///
    /// [`DecodeError::InvalidEncoding`] if any of the top three bits are set.
    pub const fn from_byte(byte: u8) -> Result<Self, DecodeError> {
        match Self::from_bits(byte) {
            Some(set) => Ok(set),
            None => Err(DecodeError::InvalidEncoding),
        }
    }
}
