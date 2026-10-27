#![cfg(feature = "aws")]

use chrono::{TimeZone, Utc};
use nexrad_data::aws::archive::Identifier;
use nexrad_data::{AwsError, Error};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Every archive file name among the repository's downloaded fixtures.
const FIXTURE_NAMES: &[&str] = &[
    "KABR20050101_000745.gz",
    "KCRP20170826_044114_V06",
    "KCRP20200101_000431_V06",
    "KCRP20210101_000031_V06",
    "KCRP20210801_000338_V06",
    "KCRP20211231_000526_V06",
    "KCRP20220101_000413_V06",
    "KDMX20220305_232324_V06",
    "KDMX20250101_225441_V06",
    "KDMX20250314_175512_V06",
    "KDMX20260101_154351_V06",
    "KLWX20180302_115347_V06",
    "KTLX19910605_162126.gz",
    "KTLX20100501_000444_V03.gz",
    "KTLX20140101_000624_V06.gz",
    "KTLX20190715_120037_V06",
];

fn parse(name: &str) -> Identifier {
    Identifier::parse(name).unwrap_or_else(|error| panic!("{name} should parse: {error}"))
}

fn hash_of(identifier: &Identifier) -> u64 {
    let mut hasher = DefaultHasher::new();
    identifier.hash(&mut hasher);
    hasher.finish()
}

fn rejection(name: &str) -> &'static str {
    match Identifier::parse(name) {
        Err(Error::Aws(AwsError::InvalidIdentifier {
            name: rejected,
            reason,
            ..
        })) => {
            assert_eq!(rejected, name);
            reason
        }
        other => panic!("{name} should be rejected, got {other:?}"),
    }
}

#[test]
fn parses_every_fixture_name() {
    for name in FIXTURE_NAMES {
        let identifier = parse(name);
        assert_eq!(identifier.name(), *name);
        assert_eq!(identifier.site(), &name[..4]);
        assert!(!identifier.is_model_data());
    }
}

#[test]
fn modern_name() {
    let identifier = parse("KDMX20220305_232324_V06");
    assert_eq!(identifier.site(), "KDMX");
    assert_eq!(
        identifier.date_time(),
        Utc.with_ymd_and_hms(2022, 3, 5, 23, 23, 24).unwrap()
    );
    assert_eq!(identifier.version(), Some("V06"));
    assert!(!identifier.is_model_data());
    assert!(!identifier.is_gzip());
}

#[test]
fn versioned_gzip_names() {
    let v03 = parse("KTLX20100501_000444_V03.gz");
    assert_eq!(v03.version(), Some("V03"));
    assert!(v03.is_gzip());
    assert_eq!(
        v03.date_time(),
        Utc.with_ymd_and_hms(2010, 5, 1, 0, 4, 44).unwrap()
    );

    let v04 = parse("KTLX20080101_000000_V04");
    assert_eq!(v04.version(), Some("V04"));
    assert!(!v04.is_gzip());
}

#[test]
fn unversioned_gzip_name() {
    let identifier = parse("KTLX19910605_162126.gz");
    assert_eq!(identifier.site(), "KTLX");
    assert_eq!(identifier.version(), None);
    assert!(identifier.is_gzip());
    assert_eq!(
        identifier.date_time(),
        Utc.with_ymd_and_hms(1991, 6, 5, 16, 21, 26).unwrap()
    );
}

#[test]
fn model_data_name() {
    let identifier = parse("KDMX20220305_232324_V06_MDM");
    assert!(identifier.is_model_data());
    assert_eq!(identifier.version(), Some("V06"));
    assert_eq!(identifier.site(), "KDMX");
    assert_eq!(
        identifier.date_time(),
        Utc.with_ymd_and_hms(2022, 3, 5, 23, 23, 24).unwrap()
    );
}

#[test]
fn site_may_contain_digits() {
    assert_eq!(parse("NOP320220305_120000_V06").site(), "NOP3");
}

#[test]
fn rejects_malformed_names() {
    for name in [
        "",
        "KDMX",
        "INVALID_NAME",
        "KDM",
        "kdmx20220305_232324_V06",
        "KDMX2022030_232324_V06",
        "KDMX20220305-232324_V06",
        "KDMX20220305_2323_V06",
        "KDMX20221301_120000_V06",
        "KDMX20220230_120000_V06",
        "KDMX20220305_256000_V06",
        "KDMX20220305_232324_V6",
        "KDMX20220305_232324_V06.bz2",
        "KDMX20220305_232324_V06_",
        "KDMX20220305_232324_MDM_V06",
        "KDMX20220305_232324_V06\u{e9}",
    ] {
        assert!(!rejection(name).is_empty());
    }
}

#[test]
fn from_str_matches_parse() {
    let parsed: Identifier = "KDMX20220305_232324_V06".parse().unwrap();
    assert_eq!(parsed, parse("KDMX20220305_232324_V06"));
    assert!("not a name".parse::<Identifier>().is_err());
}

#[test]
fn display_round_trips_through_parse() {
    for name in FIXTURE_NAMES {
        let identifier = parse(name);
        assert_eq!(identifier.to_string(), *name);
        assert_eq!(parse(&identifier.to_string()), identifier);
    }
}

#[test]
fn ordering_is_chronological_within_a_site() {
    let mut identifiers = [
        parse("KDMX20220306_120000_V06"),
        parse("KDMX20220305_130000_V06"),
        parse("KDMX20220305_120000_V06"),
    ];
    identifiers.sort();
    let times: Vec<_> = identifiers.iter().map(Identifier::date_time).collect();
    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
}

#[test]
fn equality_ordering_and_hashing_use_the_name_only() {
    let a = parse("KDMX20220305_232324_V06");
    let b = parse("KDMX20220305_232324_V06");
    let c = parse("KDMX20220305_232325_V06");

    assert_eq!(a, b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
    assert_eq!(hash_of(&a), hash_of(&b));
    assert_ne!(a, c);
    assert_eq!(a.cmp(&c), a.name().cmp(c.name()));

    // A model data object differs from its volume file only by name, and orders after it
    let model_data = parse("KDMX20220305_232324_V06_MDM");
    assert_ne!(a, model_data);
    assert_eq!(a.date_time(), model_data.date_time());
    assert!(a < model_data);
}

#[test]
fn debug_shows_the_name() {
    assert_eq!(
        format!("{:?}", parse("KDMX20220305_232324_V06")),
        "Identifier(\"KDMX20220305_232324_V06\")"
    );
}

#[test]
fn rejection_message_names_the_input() {
    let error = Identifier::parse("KDM").unwrap_err();
    assert!(error.to_string().contains("KDM"), "{error}");
}
