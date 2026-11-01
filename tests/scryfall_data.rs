//! Checks against Scryfall's own data: its symbol list and every distinct mana cost on its cards.

#[path = "helpers/fixtures.rs"]
mod fixtures;
use fixtures::fixture_lines;

use mtgcolor::{ManaCost, Symbol};

#[test]
fn every_scryfall_symbol_parses_and_prints_back_identically() {
    let rows = fixture_lines("symbology.tsv");
    assert_eq!(rows.len(), 84);
    for row in rows {
        let cols: Vec<&str> = row.split('\t').collect();
        let (text, represents_mana, in_costs) = (cols[0], cols[1] == "1", cols[2] == "1");

        let symbol: Symbol = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(symbol.to_string(), text, "round trip of {text}");

        // Anything that can be in a cost or is mana must be a pip; the rest must not be.
        assert_eq!(
            symbol.pip().is_some(),
            represents_mana || in_costs,
            "{text}"
        );
    }
}

#[test]
fn every_real_mana_cost_round_trips_in_text_and_binary() {
    let costs = fixture_lines("mana_costs.txt");
    assert!(
        costs.len() > 800,
        "fixture looks truncated: {}",
        costs.len()
    );
    for text in costs {
        let cost = ManaCost::parse(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(cost.to_string(), text);
        assert_eq!(
            ManaCost::from_bytes(&cost.to_bytes()).unwrap(),
            cost,
            "{text}"
        );
        // The query parser reads the same cost with or without braces.
        assert_eq!(ManaCost::parse_query(&text).unwrap(), cost, "{text}");
    }
}

#[test]
fn binary_is_less_than_half_the_size_of_text_for_real_costs() {
    let (mut text, mut binary) = (0, 0);
    for line in fixture_lines("mana_costs.txt") {
        let cost: ManaCost = line.parse().unwrap();
        text += line.len();
        binary += cost.to_bytes().len();
    }
    assert!(binary * 2 < text, "{binary} binary vs {text} text bytes");
}
