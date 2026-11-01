#![cfg(feature = "serde")]

use mtgcolor::{Color, ColorIdentity, ColorSet, ManaCost, Pip, Symbol};
use serde_json::{from_str, json, to_value};

#[test]
fn colors_use_scryfalls_letter_forms() {
    assert_eq!(to_value(Color::Blue).unwrap(), json!("U"));
    assert_eq!(from_str::<Color>("\"u\"").unwrap(), Color::Blue);
    assert_eq!(from_str::<Color>("\"Blue\"").unwrap(), Color::Blue);
    assert!(from_str::<Color>("\"X\"").is_err());
}

#[test]
fn color_sets_and_identities_serialize_like_scryfalls_arrays() {
    // Scryfall and MTGJSON list colors alphabetically, not in WUBRG order.
    let set: ColorSet = "grub".parse().unwrap();
    assert_eq!(to_value(set).unwrap(), json!(["B", "G", "R", "U"]));
    assert_eq!(
        from_str::<ColorSet>(r#"["G","W"]"#).unwrap(),
        "wg".parse().unwrap()
    );
    assert_eq!(from_str::<ColorSet>("[]").unwrap(), ColorSet::COLORLESS);
    assert_eq!(to_value(ColorSet::COLORLESS).unwrap(), json!([]));
    assert_eq!(
        to_value(ColorIdentity::new(set)).unwrap(),
        json!(["B", "G", "R", "U"])
    );
    // Real Scryfall arrays come back byte for byte.
    for wire in [
        r#"["G","U","W"]"#,
        r#"["B","G","R","U","W"]"#,
        r#"["R","U"]"#,
        "[]",
    ] {
        let parsed: ColorSet = from_str(wire).unwrap();
        assert_eq!(serde_json::to_string(&parsed).unwrap(), wire);
    }
    assert_eq!(
        from_str::<ColorIdentity>(r#"["R"]"#).unwrap(),
        ColorIdentity::new(ColorSet::RED)
    );
    assert!(from_str::<ColorSet>(r#"["Q"]"#).is_err());
}

#[test]
fn pips_costs_and_symbols_are_strings() {
    let cost: ManaCost = "{2}{W/U}{G/P}".parse().unwrap();
    assert_eq!(to_value(&cost).unwrap(), json!("{2}{W/U}{G/P}"));
    assert_eq!(from_str::<ManaCost>(r#""{2}{W/U}{G/P}""#).unwrap(), cost);
    assert_eq!(from_str::<ManaCost>(r#""""#).unwrap(), ManaCost::new());
    assert_eq!(to_value(Pip::Infinity).unwrap(), json!("{∞}"));
    assert_eq!(from_str::<Symbol>(r#""{T}""#).unwrap(), Symbol::Tap);
    let error = from_str::<ManaCost>(r#""{2}{Q}""#).unwrap_err().to_string();
    assert!(error.contains("unknown symbol"), "{error}");
}

#[test]
fn works_as_struct_fields_the_way_a_card_would_use_them() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Card {
        mana_cost: Option<ManaCost>,
        colors: ColorSet,
        color_identity: ColorIdentity,
    }
    let card: Card =
        from_str(r#"{"mana_cost":"{1}{U}{R}","colors":["U","R"],"color_identity":["U","R"]}"#)
            .unwrap();
    assert_eq!(card.mana_cost.as_ref().unwrap().colors(), card.colors);
    let none: Card = from_str(r#"{"mana_cost":null,"colors":[],"color_identity":["G"]}"#).unwrap();
    assert!(none.mana_cost.is_none());
    assert_eq!(
        from_str::<Card>(&serde_json::to_string(&card).unwrap()).unwrap(),
        card
    );
}
