# mtgcolor

Colors, color identity, mana pips and mana costs for Magic: The Gathering.

- **Represent** colors, color sets, color identity, individual mana symbols and whole mana costs
  as small `Copy`/`Eq`/`Hash` types.
- **Parse** Scryfall's syntax: `{2}{W/U}` costs, `c:`/`id:` color values (`esper`, `rg`,
  `quandrix`, `chaos`), `m:` shorthand (`2WW`), and every `{...}` symbol in Oracle text.
- **Compare** by containment: subset and superset on colors, "fits in this commander", and
  bag-of-symbols comparison on costs, all through Scryfall's `=`, `!=`, `<`, `<=`, `>`, `>=`.
- **Encode** compactly: a pip is one byte, a color set is one byte, `{2}{W}{W}` is four bytes.
  Every value has exactly one valid encoding, so bytes work as keys.

`no_std` (needs `alloc`), no dependencies, optional `serde`.

```rust
use mtgcolor::{Cmp, ColorIdentity, ColorSet, ManaCost};

let cost: ManaCost = "{2}{W/U}{W/U}".parse()?;
assert_eq!(cost.mana_value(), 4.0);
assert_eq!(cost.colors().to_string(), "WU");

// Commander legality: does it fit in a Bant (G/W/U) deck?
let bant: ColorIdentity = "bant".parse()?;
assert!(ColorIdentity::from_cost(&cost).fits_in(bant));

// Scryfall `m:` semantics: does the cost include these symbols?
assert!(cost.contains(&ManaCost::parse_query("{W/U}")?));
assert!(cost.compare(Cmp::Gt, &ManaCost::parse_query("2")?));

// Scryfall color expressions.
let id: ColorSet = "azorius".parse()?;
assert!(ColorSet::BLUE.compare(Cmp::Le, id)); // id<=azorius

// One byte per pip.
assert_eq!(cost.to_bytes(), [3, 0x02, 0x20, 0x20]);
# Ok::<(), mtgcolor::ParseError>(())
```

## Model

| Type | What it is |
| --- | --- |
| `Color` | W, U, B, R, G |
| `ColorSet` | any of the 32 combinations, one byte, set algebra and containment comparison |
| `ColorIdentity` | a `ColorSet` for Commander identity, built from cost, rules text, type line and indicator |
| `Pip` | one mana symbol: every symbol Scryfall documents for costs (colored, generic, X/Y/Z, hybrid, `{2/W}`, `{C/W}`, Phyrexian, `{G/U/P}`, half, `{∞}`, `{L}`, `{D}`, ...) |
| `ManaCost` | pips in printed order |
| `Symbol` | any `{...}` in Oracle text: a `Pip` or `{T}`, `{Q}`, `{E}`, `{P}`, `{PW}`, `{CHAOS}`, `{A}`, `{TK}` |
| `ColorSpec`, `Cmp` | the value and operator halves of `c:`/`id:` for building a query engine |

Design choices worth knowing:

- Hybrids are normalised: `{U/W}` is the same value as `{W/U}` and prints as Scryfall prints it.
  Invalid shapes such as `{W/W}` or `{3/W}` cannot be built or parsed.
- `Pip` and `Symbol` are `#[non_exhaustive]`. New symbols are added as they are printed.
- Parsing is strict: a symbol Scryfall does not document is an error, not a guess.
- Comparison is containment, not magnitude, and two values can be incomparable. `ColorSet`'s
  `Ord` is just the bit order so it can key a map; use `compare`/`subset_cmp` for containment.
- Display is WUBRG (`UBRG`). Serde uses Scryfall's and MTGJSON's alphabetical arrays
  (`["B","G","R","U"]`) so card JSON round-trips unchanged.
- `ManaCost::colors()` is the colors of the cost only. A card's color can also come from a color
  indicator or its own text (Devoid, "is all colors"); pass those in yourself.

## Binary format

See the table in the [`binary`](src/binary.rs) module. A pip is one byte (high nibble: kind,
low nibble: payload); generic amounts of 15 or more add a LEB128 varint. A cost is its pip count
as a varint, then the pips. On the 824 distinct costs found on 108,000 Scryfall cards the
binary form is 39% the size of the text.

Decoders reject reserved tags, out-of-range payloads, padded varints and short-form values
written long, so equal values always have equal bytes.

## How it was checked

- All 84 symbols from Scryfall's `/symbology` parse and print back identically.
- All 824 distinct `mana_cost` strings on Scryfall's cards round-trip through text and binary.
- On a deduplicated sample of 5,700 cards (`tests/fixtures/cards.tsv`) the crate reproduces
  Scryfall's `color_identity` from cost, rules text, type line and indicator for every card
  except six whose identity comes from a characteristic-defining ability. Computed over the full
  105,800-card set the same way, 17 printings (those six cards) differ.
- Card color from cost plus indicator matches Scryfall's `colors` except for Devoid cards,
  cards that set their own color, and cards with no colored cost that Scryfall still colors
  (Pacts, suspend-only cards).

## Not covered

- Colors and identity of multi-faced cards: combine the faces yourself.
- Non-Oracle symbol sets (Un-set `{½}` and `{∞}` parse, but there is no model for rules beyond
  mana value).
- `m:`-style *matching of a card* against a query; this crate gives the comparison primitives.

Mana symbols are copyright Wizards of the Coast. This crate contains no artwork and is not
produced or endorsed by Wizards of the Coast. Test fixtures contain only facts (symbol lists,
mana costs, color arrays) taken from [Scryfall](https://scryfall.com).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
