//! Colors, color identity, mana pips and mana costs for Magic: The Gathering.
//!
//! - [`Color`] and [`ColorSet`]: the five colors and the 32 combinations of them, with set
//!   algebra, containment comparison and Scryfall's color names (`azorius`, `bant`, `chaos`).
//! - [`Cmp`]: Scryfall's comparison operators, which on colors mean containment.
//! - [`ColorSpec`] and [`ColorKind`]: the value half of Scryfall's `c:` and `id:` keywords, for
//!   building a query engine on top.
//! - A one-byte binary encoding for a [`ColorSet`].
//!
//! ```
//! use mtgcolor::{Cmp, ColorSet};
//!
//! // Does a Bant (GWU) commander's deck have room for a white-blue card?
//! let commander: ColorSet = "bant".parse().unwrap();
//! let card: ColorSet = "wu".parse().unwrap();
//! assert!(card.is_subset(commander));
//! assert!(card.compare(Cmp::Lt, commander));
//!
//! // One byte instead of text.
//! let mut bytes = Vec::new();
//! card.encode(&mut bytes);
//! assert_eq!(bytes, [0b0_0011]);
//! ```
//!
//! The crate is `no_std` (it needs `alloc`).

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

mod cmp;
mod color;
mod error;
mod query;

pub use cmp::Cmp;
pub use color::{Color, ColorSet, Iter as ColorSetIter};
pub use error::{DecodeError, ParseError};
pub use query::{ColorKind, ColorSpec};
