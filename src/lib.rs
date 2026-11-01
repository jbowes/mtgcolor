//! Colors, color identity, mana pips and mana costs for Magic: The Gathering.
//!
//! - [`Color`] and [`ColorSet`]: the five colors and the 32 combinations of them, with set
//!   algebra, containment comparison and Scryfall's color names (`azorius`, `bant`, `chaos`).
//! - [`ColorIdentity`]: a card's Commander color identity, built from its cost, rules text,
//!   type line and indicator, with [`fits_in`](ColorIdentity::fits_in) for deck legality.
//! - [`Pip`] and [`ManaCost`]: every symbol Scryfall documents for mana costs, parsed from and
//!   printed to Scryfall's `{2}{W/U}` syntax, with mana value, colors, devotion and
//!   bag-of-symbols comparison.
//! - [`Symbol`] and [`Symbols`]: all `{...}` symbols in Oracle text, including `{T}` and `{E}`.
//! - [`ColorSpec`] and [`Cmp`]: the value and operator halves of Scryfall's `c:`, `id:` and `m:`
//!   keywords, for building a query engine on top.
//! - A compact, canonical binary encoding for all of the above: see [`binary`].
//!
//! ```
//! use mtgcolor::{ColorIdentity, ManaCost};
//!
//! let cost: ManaCost = "{2}{W/U}{W/U}".parse().unwrap();
//! assert_eq!(cost.mana_value(), 4.0);
//! assert_eq!(cost.colors().to_string(), "WU");
//!
//! // Does it fit in a Bant (GWU) commander's deck?
//! let commander: ColorIdentity = "bant".parse().unwrap();
//! assert!(ColorIdentity::from_cost(&cost).fits_in(commander));
//!
//! // Four bytes instead of thirteen characters.
//! assert_eq!(cost.to_bytes(), [3, 0x02, 0x20, 0x20]);
//! ```
//!
//! The crate is `no_std` (it needs `alloc`). Enable the `serde` feature for `Serialize` and
//! `Deserialize` using Scryfall's string forms.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

pub mod binary;
mod cmp;
mod color;
mod cost;
mod error;
mod identity;
mod pip;
mod query;
#[cfg(feature = "serde")]
mod serde_impls;
mod symbol;

pub use cmp::Cmp;
pub use color::{Color, ColorSet, Iter as ColorSetIter};
pub use cost::ManaCost;
pub use error::{DecodeError, ParseError};
pub use identity::ColorIdentity;
pub use pip::{ColorPair, Pip};
pub use query::{ColorKind, ColorSpec};
pub use symbol::{Symbol, Symbols};

/// Compiles and runs the README's example as a doctest.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
