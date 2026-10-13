# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.2`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](../docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root (`nexrad_model::Error`); `nexrad_model::result` remains the
  defining module. (#158)
- Builders for the types whose constructors took long positional argument lists: `Radial::builder`,
  `ElevationCut::builder`, `VolumeCoveragePattern::builder`, `SweepField::builder` and `Site::builder`, with the new
  types `data::{RadialBuilder, ElevationCutBuilder, VolumeCoveragePatternBuilder, SweepFieldBuilder}` and
  `meta::SiteBuilder`. (#165)
- `Error::InvalidSweepGeometry(String)`, returned by `SweepFieldBuilder::build`. (#165)
- `Radial::moment(Product) -> Option<&MomentData>`, a uniform accessor for every moment. (#170)
- `MomentData::product()`, `SweepField::product() -> Option<Product>` and `SweepFieldBuilder::product(Product)`.
  (#170)
- `MomentValue::CfpStatus(CfpStatus)`, produced only for `Product::ClutterFilterPower`. `MomentValue` is exhaustive, so
  exhaustive matches need a new arm. (#170)
- `Sweep::field(product)`, `Scan::fields(product)`, `Scan::first_field(product)` and `Scan::coordinate_system()`,
  which replace the facade's `extract_field`, `extract_fields`, `extract_first_field` and `coordinate_system`. (#172)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error`, `data::Product` and `data::VcpNumber` are `#[non_exhaustive]`; matches on them outside this crate need a
  wildcard arm. (#155)
- Acronyms in names follow UpperCamelCase (#160):
  - `VCPNumber` → `VcpNumber`, `CFPStatus` → `CfpStatus`.
  - `ChannelConfiguration::SZ2Phase` → `Sz2Phase`.
  - `RadialStatus::ElevationStartVCPFinal` → `ElevationStartVcpFinal`.
  - `WaveformType::CS` → `ContiguousSurveillance`, `CDW` → `ContiguousDopplerWithAmbiguityResolution`,
    `CDWO` → `ContiguousDopplerWithoutAmbiguityResolution`, `B` → `Batch`, `SPP` → `StaggeredPulsePair`.
  - With `serde`, `WaveformType`, `ChannelConfiguration` and `RadialStatus` serialize the new variant names; data saved
    with the old names (`"CS"`, `"CDW"`, `"CDWO"`, `"B"`, `"SPP"`, `"SZ2Phase"`, `"ElevationStartVCPFinal"`) no longer
    deserializes.
- Geographic coordinates and polar angles are `f64` (#164):
  - `meta::Site::latitude()`/`longitude()` return `f64`; with `uom`, `Site::height()`/`tower_height()` return
    `uom::si::f64::Length`.
  - `meta::registry::SiteEntry::{latitude, longitude}` are `f64`, and `registry::nearest_site` takes
    `(latitude: f64, longitude: f64)`.
  - `geo::PolarPoint::{azimuth_degrees, elevation_degrees}` are `f64`; `RadarCoordinateSystem::geo_to_polar` and
    `gate_location` take `f64` angles.
  - `SweepField::value_at_polar` takes `azimuth_degrees: f64`; `Scan::sweeps_at_elevation` takes
    `elevation_degrees: f64`.
  - With `uom`, `Radial::azimuth()`, `azimuth_spacing()`, `elevation_angle()` and `Sweep::elevation_angle()` return
    `uom::si::f64::Angle`.
  - `Radial`, `Sweep` and `SweepField` still store and return their `*_degrees` angles as `f32`.
- `Radial::new(..)` (14 arguments) is replaced by `Radial::builder(azimuth_number, azimuth_angle_degrees,
  elevation_number, elevation_angle_degrees)` with setters, then `.build()`. (#165)
- `ElevationCut::new(..)` (22 arguments) is replaced by `ElevationCut::builder(elevation_angle_degrees, waveform_type,
  channel_configuration)`. Note the argument order: `waveform_type` now comes before `channel_configuration`. (#165)
- `VolumeCoveragePattern::new(pattern_number: u16, ..)` is replaced by
  `VolumeCoveragePattern::builder(pattern_number: VcpNumber)`; convert a number with `VcpNumber::from_number(n)`. (#165)
- `Site::new(identifier, latitude, longitude, height_meters, tower_height_meters)` is replaced by
  `Site::builder(identifier, latitude, longitude).height_meters(..).tower_height_meters(..).build()`. (#165)
- `SweepField::new_empty(..)` is replaced by `SweepField::builder(label, unit)` with setters. `build()` returns
  `Result<SweepField>` and fails with `Error::InvalidSweepGeometry` when `azimuths` is empty, `gate_count` is 0 or
  `gate_interval_km` is not positive; `new_empty` did no validation. (#165)
- Clutter filter power is a `MomentData` like every other moment: `Radial::clutter_filter_power()` returns
  `Option<&MomentData>`, and `CfpMomentValue::Status(s)` / `CfpMomentValue::Value(v)` become
  `MomentValue::CfpStatus(s)` / `MomentValue::Value(v)`. (#170)
- The methods of the removed `DataMoment` trait are inherent methods on `MomentData` with the same names and
  signatures; remove `use nexrad_model::data::DataMoment` imports. (#170)
- `MomentData::from_fixed_point` takes the `Product` as a new first parameter. (#170)
- `RadialBuilder::moment(data)` stores a moment under `data.product()`; it replaces the per-product setters
  (`reflectivity`, `velocity`, ..., `clutter_filter_power`). (#170)
- `From<&MomentValue> for GateStatus` maps `CfpStatus(_)` to `GateStatus::NoData`. (#170)
- With `serde`, `MomentData` serializes as a struct with a `product` field (it was `serde(transparent)`), and
  `SweepField` gains a `product` field. (#170)
- The site registry matches NOAA's operational radar list: `KBHX`, `KDOX`, `KHDC`, `KLGX` and `PAPD` are added; `KLIX`
  (replaced by `KHDC`), the invalid Alaska aliases `KABC`, `KACG`, `KAIH` and `KAKC`, and a duplicate `KCCX` entry are
  removed; and the `KFDX` and `KPBZ` longitudes are corrected. `registry::site_by_id` returns `None` for the removed
  identifiers, and `registry::nearest_site` can return a different site. (#148)
- `Debug` output changed: renamed types and variants print their new names (#160); binary blobs print
  `fnv1a: 0x<16 hex digits>` instead of `sha256: "<hex>"` (#162); `Radial`/`MomentData` no longer nest an
  `inner: MomentDataBlock { .. }` and show a `product` field (#170).

### Removed

- `BinaryData` is no longer public; it appeared in no public signature. The `sha2` and `hex` dependencies are
  dropped. (#162)
- `CfpMomentData` and `CfpMomentValue` (formerly `CFPMomentData`/`CFPMomentValue`); use `MomentData` with
  `Product::ClutterFilterPower` and `MomentValue::CfpStatus`. (#170)
- The `DataMoment` trait; its methods are inherent on `MomentData`. (#170)
- `Product::moment_data(&Radial)` and `Product::cfp_moment_data(&Radial)`; use `radial.moment(product)`. (#170)
