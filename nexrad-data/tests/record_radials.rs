//! The two radial contracts of `Record`: `radials()` is all-or-nothing, `iter_radials()` reports
//! failures per radial.
#![cfg(feature = "model")]

use nexrad_data::result::Error;
use nexrad_data::volume::{File, Record};
use nexrad_decode::messages::MessageContents;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

/// The second record of the fixture, decompressed: the first record holding radials.
fn radial_record() -> Record<'static> {
    let file = File::new(TEST_NEXRAD_FILE.to_vec());
    let records = file.records().expect("records");
    records[1].decompress().expect("decompress")
}

/// Overwrites the azimuth angle of the `index`th digital radar data message in `record` with a
/// value outside [0, 360), so that the message still decodes but its radial does not convert.
fn corrupt_azimuth(record: &Record<'_>, index: usize) -> Record<'static> {
    let mut data = record.data().to_vec();
    let (offset, azimuth) = nexrad_decode::iter_messages(record.data())
        .map(|message| message.expect("message decodes"))
        .filter_map(|message| {
            let offset = message.offset_bytes();
            match message.contents() {
                MessageContents::DigitalRadarData(m) => {
                    Some((offset, m.header().azimuth_angle_degrees()))
                }
                _ => None,
            }
        })
        .nth(index)
        .expect("radar data message");

    let needle = azimuth.to_be_bytes();
    let window = &data[offset..offset + 64];
    let at = window
        .windows(4)
        .position(|bytes| bytes == needle)
        .expect("azimuth bytes in the message header");
    data[offset + at..offset + at + 4].copy_from_slice(&400.0f32.to_be_bytes());
    Record::new(data)
}

#[test]
fn iter_radials_matches_radials_on_a_valid_record() {
    let record = radial_record();
    let radials = record.radials().expect("radials");
    assert!(!radials.is_empty());

    let iterated: Vec<_> = record
        .iter_radials()
        .collect::<Result<_, _>>()
        .expect("every radial converts");
    assert_eq!(iterated, radials);
}

#[test]
fn radials_fails_whole_record_on_one_bad_radial() {
    let record = radial_record();
    let corrupted = corrupt_azimuth(&record, 3);

    match corrupted.radials() {
        Err(Error::Decode(_)) => {}
        other => panic!("expected a decode error, got {other:?}"),
    }
}

#[test]
fn iter_radials_skips_one_bad_radial_and_keeps_the_rest() {
    let record = radial_record();
    let expected = record.radials().expect("radials");
    let corrupted = corrupt_azimuth(&record, 3);

    let results: Vec<_> = corrupted.iter_radials().collect();
    assert_eq!(results.len(), expected.len());
    assert!(matches!(results[3], Err(Error::Decode(_))));

    let kept: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
    assert_eq!(kept.len(), expected.len() - 1);
    assert_eq!(kept[..3], expected[..3]);
    assert_eq!(kept[3..], expected[4..]);
}

#[test]
fn iter_radials_on_compressed_record_yields_one_error() {
    let file = File::new(TEST_NEXRAD_FILE.to_vec());
    let records = file.records().expect("records");
    let mut iter = records[1].iter_radials();
    assert!(matches!(iter.next(), Some(Err(Error::CompressedRecord))));
    assert!(iter.next().is_none());
}
