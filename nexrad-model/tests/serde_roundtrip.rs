//! Round trips and validation of the `serde` representation of model types.

#![cfg(feature = "serde")]

use nexrad_model::data::{
    CartesianField, CfpStatus, ChannelConfiguration, ElevationCut, GateStatus, MomentData, Product,
    PulseWidth, Radial, RadialStatus, Scan, Sweep, SweepField, VcpNumber, VerticalField,
    VolumeCoveragePattern, WaveformType,
};
use nexrad_model::geo::{GeoExtent, GeoPoint};
use nexrad_model::meta::Site;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::fmt::Debug;

const GATES: usize = 8;

fn moment(product: Product, word_size: u8) -> MomentData {
    let bytes_per_gate = usize::from(word_size / 8);
    let values = (0..GATES * bytes_per_gate)
        .map(|i| u8::try_from(i * 7 % 256).expect("fits in a byte"))
        .collect();
    MomentData::builder(product, values)
        .gate_count(GATES)
        .first_gate_range_meters(2125)
        .gate_interval_meters(250)
        .data_word_size(word_size)
        .scale(2.0)
        .offset(66.0)
        .build()
        .expect("valid moment data")
}

fn radial(azimuth_number: u16, elevation_number: u8, nyquist: Option<f32>) -> Radial {
    let mut builder = Radial::builder(
        azimuth_number,
        f32::from(azimuth_number) * 0.5,
        elevation_number,
        0.5 * f32::from(elevation_number),
    )
    .collection_timestamp_millis(1_700_000_000_000 + i64::from(azimuth_number))
    .azimuth_spacing_degrees(0.5)
    .radial_status(RadialStatus::IntermediateRadialData)
    .unambiguous_range_km(466.0);
    if let Some(nyquist) = nyquist {
        builder = builder.nyquist_velocity_mps(nyquist);
    }
    for (product, word_size) in [
        (Product::Reflectivity, 8),
        (Product::Velocity, 8),
        (Product::SpectrumWidth, 8),
        (Product::DifferentialReflectivity, 16),
        (Product::DifferentialPhase, 16),
        (Product::CorrelationCoefficient, 16),
        (Product::ClutterFilterPower, 8),
    ] {
        builder = builder.moment(moment(product, word_size));
    }
    builder.build().expect("valid radial")
}

fn scan() -> Scan {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(20)
        .build()
        .expect("valid site");
    let cuts = vec![
        ElevationCut::builder(
            0.5,
            WaveformType::ContiguousSurveillance,
            ChannelConfiguration::ConstantPhase,
        )
        .azimuth_rate_degrees_per_second(21.15)
        .super_resolution_half_degree_azimuth(true)
        .build()
        .unwrap(),
        ElevationCut::builder(1.0, WaveformType::Batch, ChannelConfiguration::Sz2Phase)
            .is_sails_cut(true)
            .sails_sequence_number(1)
            .build()
            .unwrap(),
    ];
    let vcp = VolumeCoveragePattern::builder(VcpNumber::new(212))
        .version(1)
        .doppler_velocity_resolution_mps(0.5)
        .pulse_width(PulseWidth::Short)
        .sails_enabled(true)
        .sails_cut_count(1)
        .elevation_cuts(cuts)
        .build()
        .unwrap();
    let sweeps = vec![
        Sweep::new(1, vec![radial(1, 1, Some(28.5)), radial(2, 1, None)]).expect("valid sweep"),
        Sweep::new(2, vec![radial(1, 2, Some(26.0)), radial(2, 2, None)]).expect("valid sweep"),
    ];
    Scan::with_site(site, vcp.pattern_number(), Some(vcp), sweeps)
}

fn sweep_field() -> SweepField {
    let mut field = SweepField::builder("Reflectivity", "dBZ")
        .product(Product::Reflectivity)
        .elevation_angle_degrees(0.5)
        .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
        .first_gate_range_km(2.125)
        .gate_interval_km(0.25)
        .gate_count(4)
        .build()
        .expect("valid sweep field");
    field.set(1, 2, 42.5, GateStatus::Valid).expect("in bounds");
    field
        .set(2, 0, 0.0, GateStatus::RangeFolded)
        .expect("in bounds");
    field
}

fn cartesian_field() -> CartesianField {
    let extent = GeoExtent {
        min: GeoPoint {
            latitude: 34.0,
            longitude: -99.0,
        },
        max: GeoPoint {
            latitude: 36.0,
            longitude: -97.0,
        },
    };
    let mut field =
        CartesianField::new("Composite Reflectivity", "dBZ", extent, 4, 3).expect("valid field");
    field.set(1, 2, 55.0, GateStatus::Valid).expect("in bounds");
    field
}

fn vertical_field() -> VerticalField {
    let mut field = VerticalField::new("Reflectivity", "dBZ", (0.0, 100.0), (0.0, 15_000.0), 4, 3)
        .expect("valid field");
    field
        .set(0, 3, 12.0, GateStatus::Filtered)
        .expect("in bounds");
    field
}

fn assert_round_trips<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let text = serde_json::to_string(value).expect("serializes to JSON");
    let from_json: T = serde_json::from_str(&text).expect("deserializes from JSON");
    assert_eq!(&from_json, value);

    let bytes = bincode::serialize(value).expect("serializes to bincode");
    let from_bincode: T = bincode::deserialize(&bytes).expect("deserializes from bincode");
    assert_eq!(&from_bincode, value);
}

fn to_json<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("serializes to JSON")
}

/// Removes the last entry of a JSON document's `statuses` array.
fn truncate_statuses(mut document: Value) -> Value {
    document["statuses"]
        .as_array_mut()
        .expect("statuses is an array")
        .pop();
    document
}

#[test]
fn scan_round_trips() {
    let scan = scan();
    let radials = scan.sweeps()[0].radials();
    assert_eq!(radials[0].nyquist_velocity_mps(), Some(28.5));
    assert_eq!(radials[1].nyquist_velocity_mps(), None);
    assert!(radials[0].clutter_filter_power().is_some());
    assert_round_trips(&scan);
}

#[test]
fn scan_without_coverage_pattern_round_trips() {
    let scan = Scan::new(VcpNumber::new(21), None, scan().sweeps().to_vec());
    assert_round_trips(&scan);

    let mut document = to_json(&scan);
    assert_eq!(document["coverage_pattern_number"], json!(21));
    assert!(document["coverage_pattern"].is_null());

    document
        .as_object_mut()
        .expect("scan serializes as an object")
        .remove("coverage_pattern");
    let parsed: Scan = serde_json::from_value(document).expect("coverage_pattern defaults");
    assert_eq!(parsed, scan);
}

#[test]
fn fields_round_trip() {
    assert_round_trips(&sweep_field());
    assert_round_trips(&cartesian_field());
    assert_round_trips(&vertical_field());
}

#[test]
fn keys_match_accessors() {
    let document = to_json(&sweep_field());
    assert!(document.get("statuses").is_some());
    assert!(document.get("status").is_none());

    let document = to_json(&moment(Product::Reflectivity, 8));
    for key in [
        "first_gate_range_meters",
        "gate_interval_meters",
        "raw_values",
    ] {
        assert!(document.get(key).is_some(), "missing `{key}`");
    }
    assert!(document["raw_values"].is_array());
}

#[test]
fn short_statuses_are_rejected() {
    let document = truncate_statuses(to_json(&sweep_field()));
    assert!(serde_json::from_value::<SweepField>(document).is_err());

    let document = truncate_statuses(to_json(&cartesian_field()));
    assert!(serde_json::from_value::<CartesianField>(document).is_err());

    let document = truncate_statuses(to_json(&vertical_field()));
    assert!(serde_json::from_value::<VerticalField>(document).is_err());
}

#[test]
fn invalid_field_geometry_is_rejected() {
    let mut document = to_json(&sweep_field());
    document["azimuth_angles_degrees"] = json!([2.0, 1.0, 0.0]);
    assert!(serde_json::from_value::<SweepField>(document).is_err());
}

#[test]
fn moment_length_mismatch_is_rejected() {
    let mut document = to_json(&moment(Product::DifferentialReflectivity, 16));
    document["raw_values"]
        .as_array_mut()
        .expect("raw_values is an array")
        .pop();
    let error = serde_json::from_value::<MomentData>(document)
        .expect_err("odd byte count for 16-bit gates");
    assert!(error.to_string().contains("length"), "{error}");

    let mut document = to_json(&moment(Product::Reflectivity, 8));
    document["gate_count"] = json!(GATES + 1);
    assert!(serde_json::from_value::<MomentData>(document).is_err());
}

#[test]
fn elevation_cut_keys_name_snr_thresholds() {
    let document = to_json(&scan());
    let cut = &document["coverage_pattern"]["elevation_cuts"][0];
    for key in [
        "reflectivity_snr_threshold_db",
        "velocity_snr_threshold_db",
        "spectrum_width_snr_threshold_db",
        "differential_reflectivity_snr_threshold_db",
        "differential_phase_snr_threshold_db",
        "correlation_coefficient_snr_threshold_db",
    ] {
        assert!(cut.get(key).is_some(), "missing `{key}`");
    }
    assert!(cut.get("reflectivity_threshold_db").is_none());
}

#[test]
fn invalid_coverage_pattern_is_rejected() {
    let mut document = to_json(&scan());
    document["coverage_pattern"]["doppler_velocity_resolution_mps"] = json!(-0.5);
    let error = serde_json::from_value::<Scan>(document).expect_err("negative resolution");
    assert!(
        error
            .to_string()
            .contains("doppler_velocity_resolution_mps"),
        "{error}"
    );
}

#[test]
fn moment_under_another_products_key_is_rejected() {
    let mut document = to_json(&radial(1, 1, None));
    document["velocity"] = document["reflectivity"].clone();
    assert!(serde_json::from_value::<Radial>(document).is_err());
}

#[test]
fn site_out_of_range_is_rejected() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .build()
        .expect("valid site");
    let mut document = to_json(&site);
    document["latitude"] = json!(95.0);
    let error = serde_json::from_value::<Site>(document).expect_err("latitude out of range");
    assert!(error.to_string().contains("latitude"), "{error}");
}

#[test]
fn site_identifier_is_a_string() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .build()
        .expect("valid site");
    let text = serde_json::to_string(&site).expect("serializes to JSON");
    assert!(text.contains(r#""identifier":"KTLX""#), "{text}");

    for identifier in [json!("KT"), json!("KTLXX"), json!("KT X"), json!("KT-X")] {
        let mut document = to_json(&site);
        document["identifier"] = identifier.clone();
        assert!(
            serde_json::from_value::<Site>(document).is_err(),
            "accepted {identifier}"
        );
    }

    let mut document = to_json(&site);
    document["identifier"] = json!([75, 84, 76, 88]);
    assert!(serde_json::from_value::<Site>(document).is_err());
}

#[test]
fn sweep_elevation_mismatch_is_rejected() {
    let mut document = to_json(&scan().sweeps()[0]);
    document["elevation_number"] = json!(3);
    assert!(serde_json::from_value::<Sweep>(document).is_err());
}

#[test]
fn radial_without_optional_keys_loads() {
    let radial = radial(1, 1, None);
    let mut document = to_json(&radial);
    let object = document.as_object_mut().expect("radial is an object");
    assert!(object.remove("nyquist_velocity_mps").is_some());
    assert!(object.remove("unambiguous_range_km").is_some());

    let loaded: Radial = serde_json::from_value(document).expect("loads without optional keys");
    assert_eq!(loaded.nyquist_velocity_mps(), None);
    assert_eq!(loaded.unambiguous_range_km(), None);
    assert_eq!(loaded.reflectivity(), radial.reflectivity());
}

#[test]
fn enums_serialize_as_variant_names() {
    assert_eq!(
        to_json(&CfpStatus::FilterNotApplied),
        json!("FilterNotApplied")
    );
    assert_eq!(to_json(&CfpStatus::Reserved(6)), json!({ "Reserved": 6 }));
    assert_eq!(to_json(&GateStatus::NoData), json!("NoData"));
    assert_eq!(to_json(&VcpNumber::new(212)), json!(212));
    assert_eq!(to_json(&VcpNumber::VCP_35), json!(35));
    assert_eq!(to_json(&WaveformType::Unknown(9)), json!({ "Unknown": 9 }));
    assert_eq!(
        to_json(&ChannelConfiguration::Unknown(3)),
        json!({ "Unknown": 3 })
    );
    assert_eq!(to_json(&PulseWidth::Unknown(1)), json!({ "Unknown": 1 }));
    assert_eq!(to_json(&RadialStatus::ScanStart), json!("ScanStart"));
    assert_round_trips(&WaveformType::Unknown(9));
    assert_round_trips(&VcpNumber::new(999));
}

#[test]
fn moment_bytes_are_a_byte_string_in_bincode() {
    let gate_count = 1000;
    let raw: Vec<u8> = (0..gate_count)
        .map(|i| u8::try_from(i % 256).expect("fits in a byte"))
        .collect();
    let moment = MomentData::builder(Product::Reflectivity, raw.clone())
        .gate_count(gate_count)
        .first_gate_range_meters(2125)
        .gate_interval_meters(250)
        .scale(2.0)
        .offset(66.0)
        .build()
        .expect("valid moment data");

    let bytes = bincode::serialize(&moment).expect("serializes to bincode");
    // The raw bytes end the encoding as one run behind a little-endian u64 length prefix.
    assert!(bytes.ends_with(&raw));
    assert!(
        bytes.len() < raw.len() + 64,
        "{} bytes encode {} raw bytes",
        bytes.len(),
        raw.len()
    );
    let prefix_end = bytes.len() - raw.len();
    assert_eq!(
        bytes[prefix_end - 8..prefix_end],
        (raw.len() as u64).to_le_bytes()
    );
}
