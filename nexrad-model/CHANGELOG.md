# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-10-27

Changes since `1.0.0-rc.2`, the last release candidate.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root; the `result` module remains. (#158)
- Builders `Radial::builder`, `ElevationCut::builder`, `VolumeCoveragePattern::builder`, `SweepField::builder` and
  `Site::builder`. (#165)
- `Radial::moment(Product)`, a uniform accessor for every moment. (#170)
- `MomentData::product()`, `SweepField::product()` and `SweepFieldBuilder::product`. (#170)
- `MomentValue::CfpStatus(CfpStatus)`, for clutter filter power only; exhaustive matches need a new arm. (#170)
- `Sweep::field`, `Scan::fields`, `Scan::first_field` and `Scan::coordinate_system`, replacing the facade's helpers.
  (#172)
- `MomentData::first_gate_range_meters()` and `gate_interval_meters()`. (#185)
- `Error` variants `InvalidGeometry`, `InvalidMomentData`, `MomentDataLength`, `InvalidSite` and `IndexOutOfBounds`;
  `Error` implements `Clone` and `PartialEq`. (#185)
- `GateStatus::Filtered`, for a gate whose valid value a processing step removed; exhaustive matches need a new arm.
  (#187)
- Per-radial `Radial::nyquist_velocity_mps()` and `unambiguous_range_km()` and their `RadialBuilder` setters, which reject values that are not finite and positive. (#188)
- `Scan::elevation_cut(&Sweep)` and `Scan::sweeps_with_waveform(WaveformType)`. (#188)
- `VcpNumber` constants for VCPs 11, 21, 121, 211 and 221, formerly `Unknown(n)`. (#188, #205)
- `WaveformType::abbreviation()`, the ICD's `CS`, `CD/W`, `CD/WO` or `B` (`None` for staggered pulse pair, which the
  ICD does not abbreviate, and for unknown values). (#205)
- `Display` for `RadialStatus`. (#205)
- `RadarCoordinateSystem::beam_height_meters` (above mean sea level) and `ground_range_km`. (#188)
- `MomentData::builder(product, raw_bytes)` and `MomentDataBuilder`, with setters `gate_count`,
  `first_gate_range_meters`, `gate_interval_meters`, `data_word_size`, `scale` and `offset`; the gate count defaults
  to the number of gates the raw values hold. (#210)
- `Error::InvalidCoveragePattern { parameter, reason }`, from `ElevationCutBuilder::build` and
  `VolumeCoveragePatternBuilder::build`. (#210)
- The site registry documents its contract: entries, including sites of other radar kinds such as TDWR, may be added
  in minor releases, and `nearest_site` returns the nearest listed site. (#210)
- `Product::ALL`, `Product::as_str()` (a stable snake_case name), `Product::data_block_name()` (the ICD block name),
  `Display` (the stable name) and `FromStr`, which accepts the stable name or the block name. (#207)
- `Error::UnknownProduct`, from parsing an unrecognised product name. (#207)

### Changed

- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` and `data::Product` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard arm. (#155)
- Acronyms in type and variant names are UpperCamelCase words (`VcpNumber`, `CfpStatus`, `Sz2Phase`,
  `ElevationStartVcpFinal`), and `WaveformType` variants have descriptive names; serde uses the new names. (#160)
- Geographic coordinates and polar angles are `f64`; the `*_degrees` angles stored on
  `Radial`, `Sweep` and `SweepField` stay `f32`. (#164)
- Builders replace the long positional constructors `Radial::new`, `ElevationCut::new` (the builder takes the waveform
  before the channel configuration), `VolumeCoveragePattern::new`, `Site::new` and `SweepField::new_empty`. (#165,
  #182)
- Clutter filter power is a `MomentData` like every other moment, and `MomentData::from_fixed_point` takes the
  `Product` first. (#170)
- `RadialBuilder::moment(data)` stores a moment under its product, replacing the per-product setters. (#170)
- `GateStatus::from` maps `MomentValue::CfpStatus(_)` to `GateStatus::NoData`. (#170)
- With `serde`, `MomentData` serializes as a struct with a `product` field, and `SweepField` gains a `product` field.
  (#170)
- The site registry matches NOAA's operational radar list: five sites added, `KLIX`, four invalid Alaska aliases and a
  duplicate `KCCX` removed, and `KFDX` and `KPBZ` corrected. (#148)
- Angle accessors use the stems `azimuth_angle_degrees` and `elevation_angle_degrees`; lengths end in `_meters` or
  `_km`, velocities in `_mps` and timestamps in `_millis`, and site heights are `site_elevation_meters` (above mean sea
  level) and `feedhorn_height_meters` (above ground level). Builder setters and serde names follow. (#182)
- `ElevationCut`'s elevation angle is `f32`, like the other stored angles. (#182)
- `meta::registry::SiteEntry` is `#[non_exhaustive]`. (#182)
- `meta::registry::SiteEntry::state` is `Option<&'static str>`, `None` for a site outside the United States; every
  current entry is `Some`. `Display` omits the state when it is `None`. (#211)
- `data::GateStatus` is `#[non_exhaustive]`, as `Filtered` and `NoData` come from processing steps and a later step
  may add a status; matches outside this crate need a wildcard arm. `MomentValue` stays exhaustive. (#211)
- `Debug` output prints renamed items under their new names (#160), binary blobs as `fnv1a: 0x...` hashes (#162),
  moments with a `product` field instead of a nested `inner` block (#170), and the fields renamed for serde (#189).
- Field indexing is checked: `get` and the row accessors return `Option`, and `set` returns `Result`. (#185)
- Constructors and builders with invariants validate them and return `Result`: `CartesianField::new`,
  `VerticalField::new`, `MomentData::from_fixed_point` (its `gate_count` is `usize` and its range parameters are in
  meters), `RadialBuilder::build`, `Sweep::new`, `SiteBuilder::build`, `SweepFieldBuilder::build` and
  `SiteEntry::to_site`. `Sweep::from_radials` and `Sweep::merge` are unchanged. (#185)
- `Site::identifier()` returns `&str`. (#185)
- `Error::ElevationMismatch` carries the `expected` and `actual` elevation numbers. (#185)
- `GateStatus` implements `From<MomentValue>` instead of `From<&MomentValue>`. (#187)
- `VcpNumber` is a `u16` newtype with `new`, `number(self)`, `From` both ways, `Ord` by number and associated constants
  named by pattern number (`Precipitation12` → `VcpNumber::VCP_12`), matching `nexrad-decode`; constants work as
  `match` patterns, and `VcpNumber::new(n)` replaces `from_number(n)` and `Unknown(n)`. `Display` prints `VCP 212`
  without a description. The serialized `u16` is unchanged. (#188, #205)
- `WaveformType`, `ChannelConfiguration` and `PulseWidth` carry the code in `Unknown(u8)`, like `RadialStatus`, so the
  model and `nexrad-decode` copies have the same variants; with `serde` an unknown value is `{"Unknown": n}`. `Display`
  of an unknown value includes the code (`Unknown (7)`). (#205)
- The `geo` module uses one documented earth model (spherical, 4/3 effective-radius refraction, antenna height above
  mean sea level) where the docs claimed WGS-84; `polar_to_geo` moves by under a metre at 300 km. (#188)
- With `serde`, the representation is stable within 1.x, and serialized field names equal accessor names. (#189)
- With `serde`, `MomentData`'s raw gate bytes serialize as a byte string; JSON still writes an array of integers, and
  both forms deserialize. (#189)
- With `serde`, `Site`'s `identifier` serializes as its four-character string, the only form that deserializes. (#189)
- With `serde`, `CfpStatus` variants serialize as their PascalCase names, like every other model enum. (#189)
- With `serde`, deserialization applies the constructors' and builders' validation and fails with its message. (#189)
- docs.rs shows the feature each feature-gated item requires. (#196)
- `VolumeCoveragePattern::number_of_elevation_cuts()` is `elevation_cut_count()`, like `sails_cut_count()` and
  `mrle_cut_count()`. (#210)
- `Sweep::from_radials(Vec<Radial>) -> Vec<Sweep>` is `Sweep::split_by_elevation_number`, since it returns several
  sweeps. (#210)
- `ElevationCut`'s threshold accessors and `ElevationCutBuilder` setters name the signal-to-noise threshold:
  `reflectivity_threshold_db` is `reflectivity_snr_threshold_db`, and likewise for velocity, spectrum width,
  differential reflectivity, differential phase and correlation coefficient. Serde keys follow. (#210)
- `ElevationCutBuilder::build` returns `Result` and rejects a non-finite elevation angle, azimuth rate or
  signal-to-noise threshold. (#210)
- `VolumeCoveragePatternBuilder::build` returns `Result` and rejects a Doppler velocity resolution that is not finite
  or is negative; an empty list of elevation cuts is accepted. (#210)
- With `serde`, `ElevationCut` and `VolumeCoveragePattern` deserialization applies the builders' validation. (#210)
- `RadarCoordinateSystem::geo_to_polar` returns `Option<PolarPoint>`: `None` when no beam at the elevation passes over
  the point, or an input is not finite, instead of a non-finite or negative range. (#210)
- The crates.io categories are `science` and `data-structures`; `parsing` is dropped. (#196)
- The published crate leaves out the fixture tests that need files from outside it and the site registry's source
  CSV. (#196)
- `Scan::coverage_pattern()` returns `Option<&VolumeCoveragePattern>`, `None` for a scan built without a volume
  coverage pattern message, and `Scan::coverage_pattern_number()` is stored on the scan, so it is present either way.
  `Scan::new` and `Scan::with_site` take the coverage pattern number and an `Option<VolumeCoveragePattern>`. With
  `serde`, `Scan` serializes a `coverage_pattern_number` field, and `coverage_pattern` is nullable and defaults to
  null when absent. (#208)
- Documentation only: the README states the supported data scope and its code blocks compile as doc tests; the
  `Product` variants describe the quantity each moment holds, and `Product::value_range()` is documented as a display
  default, stable for 1.x. (#209)

### Removed

- `BinaryData` from the public API, and the `sha2` and `hex` dependencies. (#162)
- `CFPMomentData` and `CFPMomentValue`; use `MomentData` with `Product::ClutterFilterPower`. (#170)
- The `DataMoment` trait; its methods are inherent on `MomentData`. (#170)
- `Product::moment_data` and `Product::cfp_moment_data`; use `Radial::moment`. (#170)
- `Site::identifier_string()`. (#185)
- The `uom` feature and the accessors it enabled (`Radial::{azimuth, azimuth_spacing, elevation_angle,
  nyquist_velocity, unambiguous_range}`, `Sweep::elevation_angle`, `MomentData::{first_gate_range, gate_interval}`,
  `Site::{site_elevation, feedhorn_height}`), and the `uom` dependency. `uom` is pre-1.0 and would tie this crate's
  major version to its own; use the unit-suffixed accessors (`elevation_angle_degrees()`, `gate_interval_km()`, ...).
  Unit-typed accessors may come later as a separate crate or extension trait. (#202)
- `MomentData::from_fixed_point`, whose eight positional arguments included two swappable pairs; use
  `MomentData::builder`. (#210)
- `VcpNumber::description()`, whose per-pattern summaries were operational characterisations rather than ICD
  definitions; it also disagreed with `nexrad-decode` on VCP 112, which the ICD lists with the MPDA scan strategy.
  (#205)

### Fixed

- `MomentData::iter`/`values` decode raw 0 and 1 as `BelowThreshold` and `RangeFolded` for every product but clutter
  filter power, also when `scale` is `0.0`. (#185)
- `SweepField::from_radials`/`from_radials_owned` take their geometry from the radials carrying the product and skip
  radials that would be mis-registered. (#185)
- `SweepField::value_at_polar` no longer wraps a sector or partial sweep across north. (#185)
- `RadarCoordinateSystem::geo_to_polar` is the exact inverse of `polar_to_geo`; its flat-earth range was about 3.6 km
  short at 300 km on a 19.5 degree tilt. (#188)
- `registry::nearest_site` uses great-circle distance, so it is correct across the antimeridian (it returned Guam
  instead of Nome in the Bering Sea). (#188)
