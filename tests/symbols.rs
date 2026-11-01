use mtgcolor::{Color, ParseError, Pip, Symbol, Symbols};

fn scan(text: &str) -> Vec<String> {
    Symbols::new(text)
        .map(|s| s.map_or_else(|e| format!("!{e}"), |s| s.to_string()))
        .collect()
}

#[test]
fn finds_symbols_in_oracle_text() {
    assert_eq!(
        scan("{T}, Pay {1}{W/U}: Add {G}{G}. {E}{E}"),
        ["{T}", "{1}", "{W/U}", "{G}", "{G}", "{E}", "{E}"]
    );
    assert!(scan("no symbols here").is_empty());
    assert!(scan("").is_empty());
}

#[test]
fn distinguishes_pawprints_from_phyrexian_mana() {
    assert_eq!("{P}".parse::<Symbol>().unwrap(), Symbol::Pawprint);
    assert_eq!(
        "{H}".parse::<Symbol>().unwrap(),
        Symbol::Mana(Pip::PhyrexianAny)
    );
    assert_eq!("{t}".parse::<Symbol>().unwrap(), Symbol::Tap);
    assert_eq!(
        "{W}".parse::<Symbol>().unwrap().pip(),
        Some(Pip::Colored(Color::White))
    );
    assert_eq!(Symbol::Tap.pip(), None);
}

#[test]
fn unknown_symbols_are_reported_not_skipped() {
    let items: Vec<_> = Symbols::new("{T} {FUTURE} {W}").collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[1], Err(ParseError::UnknownSymbol("FUTURE".into())));
    assert!(items[0].is_ok() && items[2].is_ok());
}

#[test]
fn ignores_braces_that_do_not_pair() {
    assert_eq!(scan("a } b {W"), Vec::<String>::new());
    assert_eq!(scan("{ {W} }"), ["{W}"]);
    assert_eq!(scan("{{R}}"), ["{R}"]);
}

#[test]
fn can_skip_reminder_text() {
    let text = "Flash ({T}: Add {C}. Nested (extra {R})) Then {G}. ({U}";
    let kept: Vec<String> = Symbols::outside_reminder_text(text)
        .map(|s| s.unwrap().to_string())
        .collect();
    assert_eq!(kept, ["{G}"]);
    // Whole-text scanning still sees them.
    assert_eq!(scan(text), ["{T}", "{C}", "{R}", "{G}", "{U}"]);
}
