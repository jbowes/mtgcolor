use mtgcolor::{Cmp, Color, ColorIdentity, ColorKind, ColorSet, ColorSpec, ParseError};
use std::cmp::Ordering;

fn set(s: &str) -> ColorSet {
    s.parse().unwrap()
}

#[test]
fn there_are_32_sets_and_each_round_trips_through_text_and_a_byte() {
    for bits in 0..32u8 {
        let colors = ColorSet::from_bits(bits).unwrap();
        assert_eq!(colors.bits(), bits);
        assert_eq!(colors.to_string().parse::<ColorSet>().unwrap(), colors);
        assert_eq!(ColorSet::from_byte(colors.to_byte()).unwrap(), colors);
        assert_eq!(colors.iter().collect::<ColorSet>(), colors);
        assert_eq!(colors.iter().len(), colors.len() as usize);
    }
    assert_eq!(ColorSet::from_bits(32), None);
    assert!(ColorSet::from_byte(0b1110_0000).is_err());
}

#[test]
fn iteration_and_display_are_wubrg_order() {
    let all: Vec<Color> = ColorSet::ALL.iter().collect();
    assert_eq!(all, Color::ALL);
    assert_eq!(set("grub").to_string(), "UBRG");
    assert_eq!(ColorSet::COLORLESS.to_string(), "C");
    assert_eq!(
        set("rg").iter().rev().collect::<Vec<_>>(),
        [Color::Green, Color::Red]
    );
}

#[test]
fn set_algebra() {
    let (wu, ub) = (set("wu"), set("ub"));
    assert_eq!(wu | ub, set("wub"));
    assert_eq!(wu & ub, ColorSet::BLUE);
    assert_eq!(wu - ub, ColorSet::WHITE);
    assert_eq!(wu ^ ub, set("wb"));
    assert_eq!(!wu, set("brg"));
    assert_eq!(!ColorSet::ALL, ColorSet::COLORLESS);
    assert!(wu.intersects(ub));
    assert!(set("wu").is_disjoint(set("brg")));
}

#[test]
fn comparison_is_containment_and_can_be_incomparable() {
    let (w, wu, ub) = (set("w"), set("wu"), set("ub"));
    assert_eq!(w.subset_cmp(wu), Some(Ordering::Less));
    assert_eq!(wu.subset_cmp(w), Some(Ordering::Greater));
    assert_eq!(wu.subset_cmp(wu), Some(Ordering::Equal));
    assert_eq!(wu.subset_cmp(ub), None);

    assert!(w.compare(Cmp::Le, wu) && w.compare(Cmp::Lt, wu));
    assert!(wu.compare(Cmp::Ge, w) && wu.compare(Cmp::Gt, w));
    assert!(wu.compare(Cmp::Eq, wu) && !wu.compare(Cmp::Lt, wu));
    // Incomparable sets are only "not equal".
    assert!(wu.compare(Cmp::Ne, ub));
    for op in [Cmp::Eq, Cmp::Lt, Cmp::Le, Cmp::Gt, Cmp::Ge] {
        assert!(!wu.compare(op, ub), "{op:?}");
    }
    // Colorless fits in everything.
    assert!(ColorSet::COLORLESS.is_subset(ColorSet::BLACK));
}

#[test]
fn scryfall_color_syntax() {
    assert_eq!(set("WUBRG"), ColorSet::ALL);
    assert_eq!(set("rg"), set("GR"));
    assert_eq!(set("Blue"), ColorSet::BLUE);
    assert_eq!(set("c"), ColorSet::COLORLESS);
    assert_eq!(set("colorless"), ColorSet::COLORLESS);
    // Guilds, colleges, shards, wedges and the four-color nicknames.
    assert_eq!(set("azorius"), set("wu"));
    assert_eq!(set("Quandrix"), set("gu"));
    assert_eq!(set("orzhov"), set("silverquill"));
    assert_eq!(set("bant"), set("gwu"));
    assert_eq!(set("jeskai"), set("urw"));
    assert_eq!(set("artifice"), set("wubr"));
    assert_eq!(set("chaos"), set("ubrg"));
    assert_eq!(set("aggression"), set("brgw"));
    assert_eq!(set("altruism"), set("rgwu"));
    assert_eq!(set("growth"), set("gwub"));

    assert_eq!("".parse::<ColorSet>(), Err(ParseError::Empty));
    assert!(matches!(
        "wx".parse::<ColorSet>(),
        Err(ParseError::UnknownColor(_))
    ));
    assert!(matches!(
        "cw".parse::<ColorSet>(),
        Err(ParseError::UnknownColor(_))
    ));
}

#[test]
fn every_named_set_names_itself() {
    for bits in 1..32u8 {
        let colors = ColorSet::from_bits(bits).unwrap();
        match colors.nickname() {
            Some(name) => assert_eq!(set(name), colors, "{name}"),
            None => assert!(colors == ColorSet::ALL, "{colors:?} has no nickname"),
        }
    }
    assert_eq!(set("wu").nickname(), Some("azorius"));
    assert_eq!(set("wb").nickname(), Some("orzhov"));
}

#[test]
fn color_spec_follows_scryfall_semantics() {
    let spec = |s: &str| ColorSpec::parse(s).unwrap();

    // `c:rg` is "at least red and green"; `id:rg` is "fits in red-green".
    assert!(spec("rg").matches_colon(ColorKind::Color, set("rgw")));
    assert!(!spec("rg").matches_colon(ColorKind::Color, set("r")));
    assert!(spec("rg").matches_colon(ColorKind::Identity, set("r")));
    assert!(!spec("rg").matches_colon(ColorKind::Identity, set("rgw")));
    // `id<=esper t:instant`, `c>=uw -c:red`
    assert!(spec("esper").matches(Cmp::Le, set("wb")));
    assert!(spec("uw").matches(Cmp::Ge, set("wub")));
    assert!(!spec("uw").matches(Cmp::Ge, set("w")));
    assert!(spec("uw").matches(Cmp::Ge, set("wub")) == set("wub").compare(Cmp::Ge, set("uw")));

    // Colorless means exactly colorless, even with `:`.
    assert!(spec("c").matches_colon(ColorKind::Color, ColorSet::COLORLESS));
    assert!(!spec("c").matches_colon(ColorKind::Color, set("r")));
    assert!(spec("c").matches_colon(ColorKind::Identity, ColorSet::COLORLESS));

    // Counts: `c=2`, `c>=3`.
    assert!(spec("2").matches(Cmp::Eq, set("wu")));
    assert!(!spec("2").matches(Cmp::Eq, set("w")));
    assert!(spec("2").matches(Cmp::Lt, set("w")));
    assert!(spec("0").matches_colon(ColorKind::Color, ColorSet::COLORLESS));

    // Multicolor.
    assert!(spec("m").matches_colon(ColorKind::Color, set("wu")));
    assert!(spec("multicolor").matches(Cmp::Ne, set("w")));
    assert!(!spec("m").matches_colon(ColorKind::Color, set("w")));

    assert!(matches!(
        ColorSpec::parse("6"),
        Err(ParseError::InvalidColorCount(_))
    ));
    assert!(ColorSpec::parse("purple").is_err());
}

#[test]
fn cmp_operators_parse_and_print() {
    for op in [Cmp::Eq, Cmp::Ne, Cmp::Lt, Cmp::Le, Cmp::Gt, Cmp::Ge] {
        assert_eq!(Cmp::parse(op.symbol()), Some(op));
    }
    assert_eq!(Cmp::parse("=="), Some(Cmp::Eq));
    assert_eq!(Cmp::parse(":"), None);
}

#[test]
fn commander_identity_rules() {
    let id = |s: &str| -> ColorIdentity { s.parse().unwrap() };
    assert!(id("c").fits_in(id("wu")));
    assert!(id("w").fits_in(id("wu")));
    assert!(!id("wb").fits_in(id("wu")));
    assert!(id("wubrg").fits_in(id("wubrg")));
    assert!(id("esper").compare(Cmp::Le, id("wubrg")));
    assert!(id("w").compare(Cmp::Lt, id("wu")));
}

#[test]
fn identity_is_assembled_from_cost_text_type_line_and_indicator() {
    let from = |cost: &str, text: &str, types: &str| {
        ColorIdentity::from_cost(&cost.parse().unwrap())
            | ColorIdentity::from_oracle_text(text)
            | ColorIdentity::from_type_line(types)
    };
    // Symbols in rules text count, reminder text does not.
    assert_eq!(
        from("{1}{U}", "{T}: Add {R}.", "Creature").to_string(),
        "UR"
    );
    assert_eq!(from("{1}", "({T}: Add {G}.)", "Artifact").to_string(), "C");
    assert_eq!(
        from("", "({T}: Add {G}.)", "Basic Land — Forest").to_string(),
        "G"
    );
    // Hybrid and Phyrexian symbols count their colors.
    assert_eq!(from("{W/U}", "Pay {B/P}.", "Creature").to_string(), "WUB");
    // Basic land types are whole words, and each gives its own color.
    assert_eq!(
        ColorIdentity::from_type_line("Land — Forest Island").to_string(),
        "UG"
    );
    assert_eq!(
        ColorIdentity::from_type_line("Creature — Plainswalker").to_string(),
        "C"
    );
    // Non-mana symbols and unknown ones are ignored.
    assert_eq!(
        ColorIdentity::from_oracle_text("{T}, {E}, {PW}{CHAOS} {nonsense}").to_string(),
        "C"
    );
    // An indicator is just a set.
    assert_eq!(ColorIdentity::new(set("b")).to_string(), "B");
}
