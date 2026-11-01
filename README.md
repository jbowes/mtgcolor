# mtgcolor

[![CI](https://github.com/jbowes/mtgcolor/actions/workflows/ci.yml/badge.svg)](https://github.com/jbowes/mtgcolor/actions/workflows/ci.yml)

Colors and color sets for Magic: The Gathering.

- **Represent** the five colors and all 32 combinations of them as small `Copy`/`Eq`/`Hash` types.
  `ColorSet` is a five-bit set with the usual set algebra.
- **Parse** Scryfall's color syntax: letters in any order (`rg`), color names, and the guild,
  college, shard, wedge and four-color nicknames (`azorius`, `quandrix`, `bant`, `chaos`).
- **Compare** by containment, as Scryfall does: subset and superset tests, and `=`, `!=`, `<`,
  `<=`, `>`, `>=` where two sets can also be incomparable. `ColorSpec` is the value half of the
  `c:` and `id:` search keywords, including counts (`c>=2`), `multicolor`, and the rule that `:`
  means "at least" for `c:` but "fits in" for `id:`.
- **Encode** compactly: a color set is a single byte.

`no_std` (needs `alloc`), no dependencies.

```rust
use mtgcolor::{Cmp, ColorKind, ColorSet, ColorSpec};

// Does a white-blue card fit in a Bant (green-white-blue) commander's deck?
let commander: ColorSet = "bant".parse()?;
let card: ColorSet = "wu".parse()?;
assert!(card.is_subset(commander));
assert!(card.compare(Cmp::Lt, commander));

// `id:rg` matches cards whose identity fits in red-green; `c:rg` needs both colors.
let spec = ColorSpec::parse("rg")?;
assert!(spec.matches_colon(ColorKind::Identity, "r".parse()?));
assert!(!spec.matches_colon(ColorKind::Color, "r".parse()?));

// One byte on the wire.
let mut bytes = Vec::new();
card.encode(&mut bytes);
assert_eq!(bytes, [0b0_0011]);
# Ok::<(), mtgcolor::ParseError>(())
```

Mana costs, color identity and a `serde` feature are planned.

Card names and symbols are property of Wizards of the Coast. This crate is not produced or
endorsed by Wizards of the Coast.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
