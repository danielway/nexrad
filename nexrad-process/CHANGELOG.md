# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.1`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- Validating constructors `filter::ThresholdFilter::new`, `MedianFilter::new` and `GaussianSmooth::new`, with kernels
  and sigmas in radials and gates, and getters for every parameter. (#175, #193)
- `Debug`, `Clone` and `PartialEq` for the processors, `Copy` and `Default` for `CompositeReflectivity`, and `Debug`
  for `SweepPipeline`. (#175)
- `Error::Model(nexrad_model::Error)` for an output a model constructor rejects; it displays only its layer, with the
  model error as its `source()`. (#185, #195)
- `CartesianGrid`, the validated output grid of a `ScanDerivedProduct`. (#193)
- `SweepPipeline::len()`, `is_empty()` and `steps()`. (#193)
- Getters for the parameters of `VerticalCrossSection`, `StormRelativeVelocity`, `StormCellDetector` and
  `CorrelationCoefficientFilter`. (#193)
- `derived::CompositeReflectivity::new()`. (#207)

### Changed

- `CompositeReflectivity` follows `RadarCoordinateSystem::geo_to_polar` returning `Option`: a sweep whose beam does
  not pass over an output cell is skipped there, as its non-finite range already caused. Examples build moments with
  `MomentData::builder`. (#210)
- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- `ScanDerivedProduct::compute` drops the `&Scan` argument and takes a `CartesianGrid` instead of an extent and
  resolution. (#175, #193)
- `ThresholdFilter`, `MedianFilter` and `GaussianSmooth` have private fields. (#175)
- Invalid kernel sizes and sigmas are reported by `new` instead of `process`, and `GaussianSmooth::new` rejects NaN
  and infinite sigmas. (#175)
- Angle accessors use the `nexrad-model` stems `azimuth_angle_degrees` and `elevation_angle_degrees`. (#182)
- `ThresholdFilter` and `CorrelationCoefficientFilter` mark the gates they remove `GateStatus::Filtered` and keep their
  values; the other processors pass those gates through or skip them. (#187)
- `derived::VerticalCrossSection::compute` takes the `RadarCoordinateSystem` and plots altitude above mean sea level
  against ground range. (#188)
- `SweepProcessor` and `ScanDerivedProduct` require `Send + Sync`. (#193)
- `derived::VerticalCrossSection::new` is replaced by a validating builder. (#164, #182, #193)
- `Error` derives `Clone` and `PartialEq`, its variants carry `&'static str` fields instead of a `String`, and
  `GeometryMismatch` and `UnexpectedProduct` are new. (#193)
- Every floating-point processor parameter is checked finite and in range; `CorrelationCoefficientFilter::new` now
  accepts a threshold of `0.0`. (#193)
- Processors check their inputs' product (`Error::UnexpectedProduct`), and label their outputs ("Composite
  Reflectivity" in dBZ; "Storm-Relative Velocity" in m/s with no product). (#193)
- `detection::StormCell::bounds()` returns a `nexrad_model::geo::GeoExtent` by value. (#193)
- `detection::StormCellDetector` and `StormCell` are renamed `ThresholdRegionDetector` and `ThresholdRegion`, and
  documented as connected regions at or above one reflectivity threshold; behaviour is unchanged. (#207)
- `derived::CompositeReflectivity` is `#[non_exhaustive]`, so it is built with `new()` or `Default` rather than a
  literal, and `compute` is an inherent method. (#207)

### Removed

- The `ScanProcessor` trait, which had no implementations. (#175)
- `detection::StormCellBounds`; use `nexrad_model::geo::GeoExtent`. (#193)
- The `pipeline` module path; `SweepPipeline` is at the crate root. (#193)
- The `ScanDerivedProduct` trait, which had one implementation; call `CompositeReflectivity::compute` directly.
  (#207)

### Fixed

- `CompositeReflectivity` and `VerticalCrossSection` no longer sample a sector or partial sweep at the far end of its
  azimuths across north. (#185)
- `CorrelationCoefficientFilter` compares the two fields' geometry, not only their azimuth count, so mis-registered
  gates are never masked by index. (#193)
- `StormRelativeVelocity` output is no longer labelled as the input velocity moment. (#193)
- `StormCellDetector` weights the centroid by linear reflectivity factor instead of dBZ, which gave a NaN or misplaced
  centroid for low thresholds. (#193)
- `MedianFilter` and `GaussianSmooth` wrap azimuth across north only when the field covers the full circle. (#193)
- `StormCellDetector` no longer merges a sector's two edges into one cell. (#193)
