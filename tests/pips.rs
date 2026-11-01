#[path = "helpers/all_pips.rs"]
mod all_pips;
use all_pips::all_pips;

use mtgcolor::{Color, ColorPair, ColorSet, ParseError, Pip};

fn pip(s: &str) -> Pip {
    s.parse().unwrap_or_else(|e| panic!("{s}: {e}"))
}

#[test]
fn every_pip_prints_and_parses_back() {
    for p in all_pips() {
        assert_eq!(pip(&p.to_string()), p, "{p}");
    }
}

#[test]
fn parsing_is_case_insensitive_and_printing_is_canonical() {
    assert_eq!(pip("{w}").to_string(), "{W}");
    assert_eq!(pip("{c/w}").to_string(), "{C/W}");
    assert_eq!(pip("{g/u/p}").to_string(), "{G/U/P}");
    assert_eq!(pip("{hw}").to_string(), "{HW}");
}

#[test]
fn hybrids_are_normalised_to_scryfalls_print_direction() {
    for (a, b) in [
        ("U", "W"),
        ("B", "W"),
        ("R", "B"),
        ("G", "B"),
        ("G", "R"),
        ("W", "R"),
    ] {
        let flipped = pip(&format!("{{{a}/{b}}}"));
        let printed = flipped.to_string();
        // Scryfall never prints these: {U/W} {B/W} {R/B} {G/B} {G/R} {W/R}.
        assert_ne!(printed, format!("{{{a}/{b}}}"));
        assert_eq!(pip(&printed), flipped);
    }
    assert_eq!(pip("{U/W}"), pip("{W/U}"));
    assert_eq!(pip("{W/R}"), pip("{R/W}"));
    assert_eq!(pip("{B/G/P}"), pip("{G/B/P}"));
    assert_eq!(ColorPair::new(Color::Red, Color::Red), None);
}

#[test]
fn the_ten_pairs_are_the_scryfall_ten() {
    let printed: Vec<String> = (0..10)
        .map(|i| {
            let p = ColorPair::from_index(i).unwrap();
            format!("{}/{}", p.first(), p.second())
        })
        .collect();
    assert_eq!(
        printed,
        ["W/U", "W/B", "B/R", "B/G", "U/B", "U/R", "R/G", "R/W", "G/W", "G/U"]
    );
    assert!(ColorPair::from_index(10).is_none());
}

#[test]
fn rejects_what_scryfall_does_not_document() {
    for bad in [
        "{}",
        "{ }",
        "{Q/W}",
        "{W/W}",
        "{3/W}",
        "{W/U/B}",
        "{W/U/B/P}",
        "{W/P/P}",
        "{P/W}",
        "{HG2}",
        "{HC}",
        "{01}",
        "{+1}",
        "{-1}",
        "{1.5}",
        "{4294967296}",
        "{ W}",
        "{W }",
        "{w/ u}",
        "{2/C}",
        "{C/C}",
        "{C/U/P}",
        "{X/W}",
    ] {
        assert!(
            matches!(Pip::parse(bad), Err(ParseError::UnknownSymbol(_))),
            "{bad}"
        );
    }
    for bad in ["W", "{W", "W}", "{{W}}", "{W}{U}", "", "{W}x"] {
        assert_eq!(Pip::parse(bad), Err(ParseError::MalformedBraces), "{bad:?}");
    }
}

#[test]
fn colors_follow_the_rules_for_card_color() {
    let colors = |s: &str| pip(s).colors().to_string();
    assert_eq!(colors("{W}"), "W");
    assert_eq!(colors("{W/U}"), "WU");
    assert_eq!(colors("{2/B}"), "B");
    assert_eq!(colors("{C/R}"), "R");
    assert_eq!(colors("{G/P}"), "G");
    assert_eq!(colors("{G/U/P}"), "UG");
    assert_eq!(colors("{HW}"), "W");
    for colorless in [
        "{3}", "{X}", "{C}", "{S}", "{C/P}", "{H}", "{½}", "{∞}", "{L}", "{D}",
    ] {
        assert_eq!(pip(colorless).colors(), ColorSet::COLORLESS, "{colorless}");
    }
}

#[test]
fn mana_value_matches_scryfalls_numbers() {
    let mv = |s: &str| pip(s).mana_value();
    assert_eq!(mv("{0}"), 0.0);
    assert_eq!(mv("{16}"), 16.0);
    assert_eq!(mv("{1000000}"), 1_000_000.0);
    assert_eq!(mv("{X}"), 0.0);
    assert_eq!(mv("{W}"), 1.0);
    assert_eq!(mv("{W/U}"), 1.0);
    assert_eq!(mv("{2/W}"), 2.0);
    assert_eq!(mv("{C/W}"), 1.0);
    assert_eq!(mv("{W/P}"), 1.0);
    assert_eq!(mv("{HW}"), 0.5);
    assert_eq!(mv("{½}"), 0.5);
    assert_eq!(mv("{L}"), 1.0);
    assert_eq!(mv("{D}"), 0.0);
    assert!(mv("{∞}").is_infinite());
    assert_eq!(pip("{∞}").mana_value_halves(), None);
}

#[test]
fn predicates_match_scryfalls_is_hybrid_and_is_phyrexian() {
    for hybrid in ["{W/U}", "{2/W}", "{C/W}", "{G/U/P}"] {
        assert!(pip(hybrid).is_hybrid(), "{hybrid}");
    }
    for phyrexian in ["{W/P}", "{C/P}", "{G/U/P}", "{H}"] {
        assert!(pip(phyrexian).is_phyrexian(), "{phyrexian}");
    }
    assert!(!pip("{W}").is_hybrid() && !pip("{W}").is_phyrexian());
    assert!(pip("{Y}").is_variable() && !pip("{1}").is_variable());
}
