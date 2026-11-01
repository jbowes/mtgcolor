//! Every variant of `Pip`, for tests that need the whole space.

use mtgcolor::{Color, ColorPair, Pip};

/// Every variant of [`Pip`], with representative generic amounts and every color or pair.
pub fn all_pips() -> Vec<Pip> {
    let mut pips = Vec::new();
    for n in (0..=20).chain([99, 100, 255, 256, 65_535, 1_000_000, u32::MAX]) {
        pips.push(Pip::Generic(n));
    }
    pips.extend([
        Pip::X,
        Pip::Y,
        Pip::Z,
        Pip::Colorless,
        Pip::Snow,
        Pip::PhyrexianColorless,
        Pip::PhyrexianAny,
        Pip::HalfGeneric,
        Pip::Infinity,
        Pip::Legendary,
        Pip::LandDrop,
    ]);
    for color in Color::ALL {
        pips.extend([
            Pip::Colored(color),
            Pip::Monohybrid(color),
            Pip::ColorlessHybrid(color),
            Pip::Phyrexian(color),
            Pip::Half(color),
        ]);
    }
    for index in 0..10 {
        let pair = ColorPair::from_index(index).unwrap();
        pips.extend([Pip::Hybrid(pair), Pip::PhyrexianHybrid(pair)]);
    }
    pips
}
