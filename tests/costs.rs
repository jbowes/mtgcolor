use mtgcolor::{Cmp, Color, ColorSet, ManaCost, ParseError, Pip};
use std::cmp::Ordering;

fn cost(s: &str) -> ManaCost {
    s.parse().unwrap_or_else(|e| panic!("{s}: {e}"))
}

#[test]
fn empty_cost_is_valid_and_distinct_from_zero() {
    let none = cost("");
    assert!(none.is_empty());
    assert_eq!(none.to_string(), "");
    assert_eq!(none.mana_value(), 0.0);
    assert_ne!(none, cost("{0}"));
}

#[test]
fn colors_and_mana_value() {
    let c = cost("{2}{W/U}{W/U}{B/P}");
    assert_eq!(c.colors().to_string(), "WUB");
    assert_eq!(c.mana_value(), 5.0);
    assert_eq!(cost("{X}{X}{R}").mana_value(), 1.0);
    assert_eq!(cost("{X}{X}{R}").x_count(), 2);
    assert_eq!(cost("{HW}{HW}").mana_value(), 1.0);
    assert_eq!(cost("{HW}").mana_value_halves(), Some(1));
    assert!(cost("{∞}{R}").mana_value().is_infinite());
    assert_eq!(cost("{1000000}").mana_value(), 1_000_000.0);
    assert_eq!(cost("{W}{U}{B}{R}{G}").colors(), ColorSet::ALL);
    assert_eq!(cost("{7}").colors(), ColorSet::COLORLESS);
}

#[test]
fn devotion_counts_hybrid_toward_both_colors() {
    let c = cost("{2}{B}{B}{B/G}{G}");
    assert_eq!(c.devotion(Color::Black), 3);
    assert_eq!(c.devotion(Color::Green), 2);
    assert_eq!(c.devotion(Color::Red), 0);
    assert!(cost("{W/U}").has_hybrid() && !cost("{W}").has_hybrid());
    assert!(cost("{R/P}").has_phyrexian() && !cost("{R}").has_phyrexian());
}

#[test]
fn parse_is_strict_about_shape() {
    assert_eq!(
        ManaCost::parse("{2}x"),
        Err(ParseError::UnexpectedText("x".into()))
    );
    assert_eq!(
        ManaCost::parse("x{2}"),
        Err(ParseError::UnexpectedText("x".into()))
    );
    assert_eq!(ManaCost::parse("{2}{"), Err(ParseError::MalformedBraces));
    assert_eq!(ManaCost::parse("{2}}"), Err(ParseError::MalformedBraces));
    assert_eq!(ManaCost::parse("{{2}}"), Err(ParseError::MalformedBraces));
    assert!(matches!(
        ManaCost::parse("{2} {W}"),
        Err(ParseError::UnexpectedText(_))
    ));
    assert!(matches!(
        ManaCost::parse("{2}{Q}"),
        Err(ParseError::UnknownSymbol(_))
    ));
    // A split-card line is two costs, not one.
    assert!(ManaCost::parse("{1}{B} // {1}{U}").is_err());
}

#[test]
fn query_syntax_allows_shorthand() {
    let q = |s: &str| ManaCost::parse_query(s).unwrap_or_else(|e| panic!("{s}: {e}"));
    assert_eq!(q("2WW"), cost("{2}{W}{W}"));
    assert_eq!(q("2 w w"), cost("{2}{W}{W}"));
    assert_eq!(q("10GG"), cost("{10}{G}{G}"));
    assert_eq!(q("{2/W}{G}"), cost("{2/W}{G}"));
    assert_eq!(q("2{G/W}"), cost("{2}{G/W}"));
    assert_eq!(q("XXR"), cost("{X}{X}{R}"));
    assert_eq!(q(""), ManaCost::new());
    assert_eq!(q("  "), ManaCost::new());
    // Split symbols need their braces, as on Scryfall.
    assert!(ManaCost::parse_query("2/W").is_err());
    assert!(ManaCost::parse_query("W/U").is_err());
    assert_eq!(
        ManaCost::parse_query("{W"),
        Err(ParseError::MalformedBraces)
    );
    assert_eq!(
        ManaCost::parse_query("W}"),
        Err(ParseError::MalformedBraces)
    );
    assert!(ManaCost::parse_query("2Q").is_err());
}

#[test]
fn comparison_treats_costs_as_bags_of_symbols() {
    let (a, b) = (cost("{1}{G}{G}"), cost("{G}{G}"));
    assert_eq!(b.subset_cmp(&a), Some(Ordering::Less));
    assert_eq!(a.subset_cmp(&b), Some(Ordering::Greater));
    assert!(a.contains(&b));
    assert!(!b.contains(&a));

    // Order does not matter, repeats do.
    assert_eq!(cost("{G}{1}{G}").subset_cmp(&a), Some(Ordering::Equal));
    assert!(!cost("{1}{G}").contains(&b));
    assert!(cost("{1}{G}{G}{G}").compare(Cmp::Gt, &a));

    // Neither contains the other.
    let (g, w) = (cost("{1}{G}"), cost("{1}{W}"));
    assert_eq!(g.subset_cmp(&w), None);
    assert!(g.compare(Cmp::Ne, &w) && !g.compare(Cmp::Le, &w) && !g.compare(Cmp::Ge, &w));

    // Everything contains the empty cost.
    assert!(a.contains(&ManaCost::new()));
    // `m:{G/W}` only matches a literal {G/W}, never plain {G}.
    assert!(!cost("{G}").contains(&cost("{G/W}")));
    assert!(cost("{G/W}{G/W}").contains(&cost("{W/G}")));
}

#[test]
fn canonical_ignores_order() {
    assert_eq!(cost("{G}{2}{W}").canonical(), cost("{W}{G}{2}").canonical());
    assert_ne!(cost("{G}{2}{W}"), cost("{W}{G}{2}"));
}

#[test]
fn collects_from_and_into_pips() {
    let c: ManaCost = [Pip::Generic(2), Pip::Colored(Color::Red)]
        .into_iter()
        .collect();
    assert_eq!(c.to_string(), "{2}{R}");
    assert_eq!(c.iter().count(), 2);
    assert_eq!(c.clone().into_iter().last(), Some(Pip::Colored(Color::Red)));
    let mut d = ManaCost::new();
    d.push(Pip::X);
    d.extend(c.iter().copied());
    assert_eq!(d.to_string(), "{X}{2}{R}");
}
