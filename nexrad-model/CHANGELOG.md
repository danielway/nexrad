# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.2`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

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
  - `meta::Site::latitude()`/`longitude()` return `f64`; with `uom`, `Site::site_elevation()`/`feedhorn_height()`
    (formerly `height()`/`tower_height()`) return `uom::si::f64::Length`.
  - `meta::registry::SiteEntry::{latitude, longitude}` are `f64`, and `registry::nearest_site` takes
    `(latitude: f64, longitude: f64)`.
  - `geo::PolarPoint::{azimuth_degrees, elevation_degrees}` are `f64`; `RadarCoordinateSystem::geo_to_polar` and
    `gate_location` take `f64` angles.
  - `SweepField::value_at_polar` takes `azimuth_degrees: f64`.
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
  `Site::builder(identifier, latitude, longitude).site_elevation_meters(..).feedhorn_height_meters(..).build()`.
  (#165, #182)
- `SweepField::new_empty(..)` is replaced by `SweepField::builder(label, unit)` with setters. `build()` returns
  `Result<SweepField>` and fails with `Error::InvalidSweepGeometry` when `azimuth_angles_degrees` is empty,
  `gate_count` is 0 or `gate_interval_km` is not positive; `new_empty` did no validation. (#165)
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
  removed; and the `KFDX` and `KPBZ` coordinates are corrected. `registry::site_by_id` returns `None` for the removed
  identifiers, and `registry::nearest_site` can return a different site. (#148)
- Model accessors follow one naming rule: angles use the stems `azimuth_angle_degrees`/`elevation_angle_degrees`,
  lengths end in `_meters` or `_km`, velocities in `_mps` and timestamps in `_millis`. The matching builder setters and
  serde field names change with them (#182):
  - `SweepField::elevation_degrees()` → `elevation_angle_degrees()`, `azimuths()` → `azimuth_angles_degrees()` and
    `radial_status_mut(usize)` → `radial_statuses_mut(usize)`.
  - `geo::PolarPoint::{azimuth_degrees, elevation_degrees}` → `{azimuth_angle_degrees, elevation_angle_degrees}`.
  - `Radial::collection_timestamp()` → `collection_timestamp_millis()` (milliseconds since the Unix epoch, UTC).
  - `VolumeCoveragePattern::doppler_velocity_resolution()` → `doppler_velocity_resolution_mps()`, `sails_cuts()` →
    `sails_cut_count()` and `mrle_cuts()` → `mrle_cut_count()`.
  - `meta::Site::height_meters()` → `site_elevation_meters()` (above mean sea level) and `tower_height_meters()` →
    `feedhorn_height_meters()` (above ground level); with `uom`, `height()` → `site_elevation()` and
    `tower_height()` → `feedhorn_height()`.
  - `meta::registry::SiteEntry::elevation_meters` → `site_elevation_meters`.
- `ElevationCut::elevation_angle_degrees()` returns `f32` and `ElevationCut::builder` takes an `f32` elevation angle,
  like the angles on `Radial`, `Sweep` and `SweepField`. `Scan::sweeps_at_elevation` takes `f32` (as in
  1.0.0-rc.2). (#182)
- `meta::registry::SiteEntry` is `#[non_exhaustive]`; it can no longer be built with a struct literal outside this
  crate. (#182)
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
