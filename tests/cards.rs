//! Compares the crate's color logic with Scryfall's own `colors` and `color_identity`, on a
//! deduplicated sample of about 5,700 single-faced cards.

#[path = "helpers/fixtures.rs"]
mod fixtures;
use fixtures::fixture_lines;

use mtgcolor::{ColorIdentity, ColorSet, ManaCost};

struct Card {
    name: String,
    cost_text: String,
    colors: ColorSet,
    identity: ColorSet,
    indicator: ColorSet,
    type_line: String,
    text: String,
}

fn json_colors(json: &str) -> ColorSet {
    let letters: String = json.chars().filter(char::is_ascii_alphabetic).collect();
    if letters == "null" || letters.is_empty() {
        ColorSet::COLORLESS
    } else {
        letters.parse().unwrap_or_else(|e| panic!("{json}: {e}"))
    }
}

fn cards() -> Vec<Card> {
    fixture_lines("cards.tsv")
        .iter()
        .map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            Card {
                name: f[0].to_owned(),
                cost_text: f[2].to_owned(),
                colors: json_colors(f[3]),
                identity: json_colors(f[4]),
                indicator: json_colors(f[5]),
                type_line: f[6].to_owned(),
                text: f.get(7).copied().unwrap_or_default().to_owned(),
            }
        })
        .collect()
}

#[test]
fn every_card_cost_parses_and_round_trips_in_binary() {
    let cards = cards();
    assert!(
        cards.len() > 5_000,
        "fixture looks truncated: {}",
        cards.len()
    );
    for card in &cards {
        let cost = ManaCost::parse(&card.cost_text)
            .unwrap_or_else(|e| panic!("{}: {:?}: {e}", card.name, card.cost_text));
        assert_eq!(cost.to_string(), card.cost_text, "{}", card.name);
        assert_eq!(
            ManaCost::from_bytes(&cost.to_bytes()).unwrap(),
            cost,
            "{}",
            card.name
        );
    }
}

/// Cards whose color is set by their own rules text ("is all colors", "is colorless").
const COLOR_SET_BY_TEXT: [&str; 4] = [
    "Fallaji Wayfarer",
    "Transguild Courier",
    "Ghostfire",
    "Sphinx of the Guildpact",
];

#[test]
fn card_color_is_cost_plus_indicator_except_where_the_card_overrides_it() {
    let mut explained = 0;
    for card in cards() {
        let cost: ManaCost = card.cost_text.parse().unwrap();
        if cost.colors() | card.indicator == card.colors {
            continue;
        }
        let reason = if card.text.starts_with("Devoid") {
            "Devoid"
        } else if COLOR_SET_BY_TEXT
            .iter()
            .any(|name| card.name.contains(name))
        {
            "color-setting text"
        } else if cost.is_empty() || card.cost_text == "{0}" {
            // Pacts, suspend-only cards: Scryfall reports a color but no indicator.
            "no colored cost"
        } else {
            panic!(
                "{}: cost {} + indicator {} gives {}, Scryfall says {}",
                card.name,
                card.cost_text,
                card.indicator,
                cost.colors() | card.indicator,
                card.colors
            );
        };
        explained += usize::from(!reason.is_empty());
    }
    assert!(
        explained > 50,
        "expected Devoid and Pact cards in the sample, saw {explained}"
    );
}

/// Cards whose identity comes from a characteristic-defining ability, which no symbol shows.
const IDENTITY_SET_BY_ABILITY: [&str; 6] = [
    "Crimson Kobolds",
    "Crookshank Kobolds",
    "Kobolds of Kher Keep",
    "Rograkh, Son of Rohgahh",
    "Sphinx of the Guildpact",
    "Transguild Courier",
];

#[test]
fn identity_from_cost_text_type_line_and_indicator_matches_scryfall() {
    let mut exceptions = Vec::new();
    let mut reminder_only = 0;
    for card in cards() {
        let cost: ManaCost = card.cost_text.parse().unwrap();
        // Colored symbols that appear only inside reminder text must not leak into identity.
        let anywhere = mtgcolor::Symbols::new(&card.text)
            .filter_map(Result::ok)
            .filter_map(mtgcolor::Symbol::pip)
            .fold(ColorSet::COLORLESS, |set, pip| set | pip.colors());
        if !anywhere.is_subset(ColorIdentity::from_oracle_text(&card.text).colors()) {
            reminder_only += 1;
        }
        let ours = ColorIdentity::from_cost(&cost)
            | ColorIdentity::from_oracle_text(&card.text)
            | ColorIdentity::from_type_line(&card.type_line)
            | ColorIdentity::new(card.indicator);
        if ours.colors() != card.identity {
            assert!(
                IDENTITY_SET_BY_ABILITY.contains(&card.name.as_str()),
                "{} [{}]: ours {ours}, Scryfall {}",
                card.name,
                card.text,
                card.identity
            );
            exceptions.push(card.name);
        }
    }
    assert!(exceptions.len() <= IDENTITY_SET_BY_ABILITY.len());
    assert!(
        reminder_only > 20,
        "sample should exercise reminder text, saw {reminder_only}"
    );
}
