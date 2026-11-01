//! `Serialize` and `Deserialize` using Scryfall's string forms.
//!
//! - [`Color`]: `"W"`; deserializing also takes a name such as `"white"`.
//! - [`ColorSet`] and [`ColorIdentity`]: `["U", "W"]`, in the alphabetical order Scryfall and
//!   MTGJSON use for their `colors` and `color_identity` fields, so card JSON round-trips
//!   unchanged. Deserializing takes the letters in any order.
//! - [`Pip`], [`ManaCost`] and [`Symbol`]: `"{W/U}"`, `"{2}{W}{W}"`, `"{T}"`.
//!
//! These are text forms for JSON and the like. For the compact bytes, see [`crate::binary`].

use core::fmt;
use core::marker::PhantomData;
use core::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};

use crate::{Color, ColorIdentity, ColorSet, ManaCost, Pip, Symbol};

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(self.letter())
    }
}

/// Visits a string and parses it with `FromStr`.
struct FromStrVisitor<T> {
    expecting: &'static str,
    marker: PhantomData<T>,
}

impl<T> Visitor<'_> for FromStrVisitor<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expecting)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
        v.parse().map_err(E::custom)
    }
}

fn deserialize_from_str<'de, T, D>(deserializer: D, expecting: &'static str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    deserializer.deserialize_str(FromStrVisitor {
        expecting,
        marker: PhantomData,
    })
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_from_str(
            deserializer,
            "a color letter such as \"W\", or a color name",
        )
    }
}

impl Serialize for ColorSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter_alphabetical())
    }
}

struct ColorSetVisitor;

impl<'de> Visitor<'de> for ColorSetVisitor {
    type Value = ColorSet;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a list of color letters such as [\"W\", \"U\"]")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<ColorSet, A::Error> {
        let mut set = ColorSet::COLORLESS;
        while let Some(color) = seq.next_element::<Color>()? {
            set = set.with(color);
        }
        Ok(set)
    }
}

impl<'de> Deserialize<'de> for ColorSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(ColorSetVisitor)
    }
}

impl Serialize for ColorIdentity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.colors().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ColorIdentity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        ColorSet::deserialize(deserializer).map(Self::new)
    }
}

macro_rules! string_form {
    ($ty:ty, $expecting:literal) => {
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserialize_from_str(deserializer, $expecting)
            }
        }
    };
}

string_form!(Pip, "a mana symbol such as \"{W/U}\"");
string_form!(ManaCost, "a mana cost such as \"{2}{W}{W}\"");
string_form!(Symbol, "a symbol such as \"{T}\" or \"{W}\"");
