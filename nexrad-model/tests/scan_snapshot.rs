//! Snapshot tests for the scan decoded from `downloads/KDMX20220305_232324_V06`.
//!
//! Rather than serializing the whole scan (which produces a multi-megabyte snapshot), these
//! tests capture the site and coverage pattern in full, a compact per-sweep digest of every
//! decoded moment, and the full contents of a small fixed sample of radials.

#![cfg(feature = "serde")]

use insta::assert_yaml_snapshot;
use nexrad_data::volume;
use nexrad_model::data::{MomentValue, Product, Scan, Sweep};
use serde::Serialize;
use sha2::{Digest, Sha256};

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

/// Every product, in a fixed order, for per-sweep digests.
const PRODUCTS: [Product; 7] = [
    Product::Reflectivity,
    Product::Velocity,
    Product::SpectrumWidth,
    Product::DifferentialReflectivity,
    Product::DifferentialPhase,
    Product::CorrelationCoefficient,
    Product::ClutterFilterPower,
];

/// Number of leading sweeps whose first radial is snapshotted in full.
const SAMPLED_SWEEPS: usize = 3;

fn load_scan() -> Scan {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    volume.scan().expect("decodes into scan model")
}

#[test]
fn test_decode_map_scan_site_and_coverage_pattern() {
    let scan = load_scan();
    assert_yaml_snapshot!(
        "decode_map_scan_site_and_coverage_pattern",
        (scan.site(), scan.coverage_pattern())
    );
}

#[derive(Serialize)]
struct SweepDigest {
    elevation_number: u8,
    elevation_angle_degrees: Option<f32>,
    radial_count: usize,
    products: Vec<ProductDigest>,
}

#[derive(Serialize)]
struct ProductDigest {
    product: Product,
    radial_count: usize,
    /// Distinct gate geometries seen across the sweep's radials, in first-seen order.
    geometries: Vec<GateGeometry>,
    /// SHA-256 of every decoded gate value and status, radial by radial.
    values_sha256: String,
}

#[derive(Serialize, PartialEq)]
struct GateGeometry {
    first_gate_range_km: f64,
    gate_interval_km: f64,
    gate_count: u16,
}

fn digest_sweep(sweep: &Sweep) -> SweepDigest {
    let products = PRODUCTS
        .iter()
        .filter_map(|&product| {
            let mut radial_count = 0;
            let mut geometries = Vec::new();
            let mut hash = Sha256::new();

            for moment in sweep.radials().iter().filter_map(|r| r.moment(product)) {
                radial_count += 1;

                let geometry = GateGeometry {
                    first_gate_range_km: moment.first_gate_range_km(),
                    gate_interval_km: moment.gate_interval_km(),
                    gate_count: moment.gate_count(),
                };
                if !geometries.contains(&geometry) {
                    geometries.push(geometry);
                }

                hash.update(moment.gate_count().to_le_bytes());
                for value in moment.iter() {
                    hash_moment_value(&mut hash, value);
                }
            }

            (radial_count > 0).then(|| ProductDigest {
                product,
                radial_count,
                geometries,
                values_sha256: hex::encode(hash.finalize()),
            })
        })
        .collect();

    SweepDigest {
        elevation_number: sweep.elevation_number(),
        elevation_angle_degrees: sweep.elevation_angle_degrees(),
        radial_count: sweep.radials().len(),
        products,
    }
}

/// Feeds one decoded gate into the hash as a tag byte (status) plus the value's bits.
fn hash_moment_value(hash: &mut Sha256, value: MomentValue) {
    match value {
        MomentValue::Value(value) => {
            hash.update([0]);
            hash.update(value.to_bits().to_le_bytes());
        }
        MomentValue::BelowThreshold => hash.update([1]),
        MomentValue::RangeFolded => hash.update([2]),
        MomentValue::CfpStatus(status) => {
            hash.update([3]);
            hash.update(format!("{status:?}").as_bytes());
        }
    }
}

#[test]
fn test_decode_map_scan_sweep_digests() {
    let scan = load_scan();
    let digests: Vec<SweepDigest> = scan.sweeps().iter().map(digest_sweep).collect();
    assert_yaml_snapshot!("decode_map_scan_sweep_digests", digests);
}

#[test]
fn test_decode_map_scan_sample_radials() {
    let scan = load_scan();
    let radials: Vec<_> = scan
        .sweeps()
        .iter()
        .take(SAMPLED_SWEEPS)
        .map(|sweep| sweep.radials().first().expect("sweep has radials"))
        .collect();
    assert_eq!(radials.len(), SAMPLED_SWEEPS);

    assert_yaml_snapshot!("decode_map_scan_sample_radials", radials, {
      ".**.values" => insta::dynamic_redaction(|value, _path| redact_bytes_like(value)),
    });
}

fn redact_bytes_like(value: insta::internals::Content) -> insta::internals::Content {
    if let Some(items) = value.as_slice() {
        let mut buf = Vec::with_capacity(items.len());
        for item in items {
            if let Some(n) = item.as_u64() {
                if n <= 255 {
                    buf.push(n as u8);
                    continue;
                }
            }
            panic!("unexpected value in binary data: {:?}", item);
        }
        return hash_bytes_to_content(&buf);
    }
    value
}

fn hash_bytes_to_content(bytes: &[u8]) -> insta::internals::Content {
    let mut hash = Sha256::new();
    hash.update(bytes);

    let digest = hex::encode(hash.finalize());
    format!("bytes::<{}>::sha256:{digest}", bytes.len()).into()
}
