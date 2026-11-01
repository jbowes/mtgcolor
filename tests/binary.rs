#[path = "helpers/all_pips.rs"]
mod all_pips;
use all_pips::all_pips;

use mtgcolor::{ColorIdentity, ColorSet, DecodeError, ManaCost, Pip};

#[test]
fn every_pip_round_trips_and_has_one_encoding() {
    let mut seen = std::collections::HashMap::new();
    for pip in all_pips() {
        let mut bytes = Vec::new();
        pip.encode(&mut bytes);
        let (decoded, used) = Pip::decode(&bytes).unwrap();
        assert_eq!((decoded, used), (pip, bytes.len()), "{pip}");
        if let Some(other) = seen.insert(bytes.clone(), pip) {
            panic!("{pip} and {other} share the encoding {bytes:02x?}");
        }
    }
}

#[test]
fn encoding_sizes() {
    let size = |pip: Pip| {
        let mut b = Vec::new();
        pip.encode(&mut b);
        b.len()
    };
    assert_eq!(size(Pip::Generic(0)), 1);
    assert_eq!(size(Pip::Generic(14)), 1);
    assert_eq!(size(Pip::Generic(15)), 2);
    assert_eq!(size(Pip::Generic(127)), 2);
    assert_eq!(size(Pip::Generic(128)), 3);
    assert_eq!(size(Pip::Generic(1_000_000)), 4);
    assert_eq!(size(Pip::Generic(u32::MAX)), 6);
    assert_eq!(size("{G/U/P}".parse().unwrap()), 1);
}

#[test]
fn documented_example() {
    let cost: ManaCost = "{2}{W/U}{W/U}".parse().unwrap();
    assert_eq!(cost.to_bytes(), [3, 0x02, 0x20, 0x20]);
    assert_eq!(ManaCost::new().to_bytes(), [0]);
}

#[test]
fn costs_round_trip_and_report_how_much_they_used() {
    let cost: ManaCost = "{X}{X}{1000000}{G/U/P}{HW}{∞}".parse().unwrap();
    let mut bytes = cost.to_bytes();
    let len = bytes.len();
    bytes.extend([0xAA, 0xBB]);
    assert_eq!(ManaCost::decode(&bytes).unwrap(), (cost.clone(), len));
    assert_eq!(
        ManaCost::from_bytes(&bytes),
        Err(DecodeError::TrailingBytes)
    );
    assert_eq!(ManaCost::from_bytes(&bytes[..len]).unwrap(), cost);
}

#[test]
fn rejects_reserved_out_of_range_and_truncated_input() {
    // Reserved kinds and out-of-range payloads.
    for bad in [0xA0, 0xF0, 0x15, 0x1F, 0x2A, 0x6A, 0x75, 0x83, 0x98, 0x9F] {
        assert_eq!(
            Pip::decode(&[bad]),
            Err(DecodeError::InvalidEncoding),
            "{bad:#04x}"
        );
    }
    assert_eq!(Pip::decode(&[]), Err(DecodeError::UnexpectedEof));
    assert_eq!(Pip::decode(&[0x0F]), Err(DecodeError::UnexpectedEof));
    assert_eq!(Pip::decode(&[0x0F, 0x80]), Err(DecodeError::UnexpectedEof));
    assert_eq!(ManaCost::decode(&[]), Err(DecodeError::UnexpectedEof));
    assert_eq!(
        ManaCost::decode(&[2, 0x10]),
        Err(DecodeError::UnexpectedEof)
    );
}

#[test]
fn rejects_non_canonical_forms_so_each_value_has_one_encoding() {
    // Generic(3) must be one byte, not escaped.
    assert_eq!(Pip::decode(&[0x0F, 0x03]), Err(DecodeError::NonCanonical));
    assert_eq!(Pip::decode(&[0x0F, 0x0E]), Err(DecodeError::NonCanonical));
    assert_eq!(Pip::decode(&[0x0F, 0x0F]).unwrap().0, Pip::Generic(15));
    // Padded varints.
    assert_eq!(
        Pip::decode(&[0x0F, 0x8F, 0x00]),
        Err(DecodeError::NonCanonical)
    );
    assert_eq!(
        ManaCost::decode(&[0x80, 0x00]),
        Err(DecodeError::NonCanonical)
    );
}

#[test]
fn rejects_generic_amounts_beyond_u32_and_varints_beyond_u64() {
    let too_big_for_u32 = [0x0F, 0x80, 0x80, 0x80, 0x80, 0x10];
    assert_eq!(
        Pip::decode(&too_big_for_u32),
        Err(DecodeError::InvalidEncoding)
    );
    let endless = [0xFF; 11];
    assert_eq!(
        ManaCost::decode(&endless),
        Err(DecodeError::InvalidEncoding)
    );
}

#[test]
fn a_hostile_count_cannot_make_decode_allocate() {
    // Claims 2^60 pips in a few bytes.
    let mut bytes = vec![0x80; 8];
    bytes.push(0x10);
    assert_eq!(ManaCost::decode(&bytes), Err(DecodeError::UnexpectedEof));
}

#[test]
fn color_sets_are_one_byte() {
    for bits in 0..32u8 {
        let set = ColorSet::from_bits(bits).unwrap();
        let mut out = Vec::new();
        set.encode(&mut out);
        assert_eq!(out, [bits]);
        assert_eq!(ColorSet::decode(&out).unwrap(), (set, 1));
        assert_eq!(ColorIdentity::from_byte(bits).unwrap().to_byte(), bits);
    }
    assert_eq!(ColorSet::decode(&[]), Err(DecodeError::UnexpectedEof));
    assert_eq!(ColorSet::decode(&[0x20]), Err(DecodeError::InvalidEncoding));
}

#[test]
fn decode_never_panics_on_arbitrary_bytes() {
    // A deterministic scramble; not a fuzzer, but covers short and long garbage.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    for len in 0..64 {
        for _ in 0..200 {
            let bytes: Vec<u8> = (0..len)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    (state >> 24) as u8
                })
                .collect();
            if let Ok((cost, used)) = ManaCost::decode(&bytes) {
                // Whatever decodes must re-encode to exactly the bytes it came from.
                assert_eq!(cost.to_bytes(), bytes[..used]);
            }
            let _ = Pip::decode(&bytes);
            let _ = ColorSet::decode(&bytes);
        }
    }
}
