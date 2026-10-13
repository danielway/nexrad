# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.1`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- Validating constructors `filter::ThresholdFilter::new(min, max)`, `filter::MedianFilter::new(azimuth_kernel,
  range_kernel)` and `filter::GaussianSmooth::new(sigma_azimuth, sigma_range)`, each returning
  `Err(Error::InvalidParameter)` for invalid parameters, with getters for every parameter (`min()`, `max()`,
  `azimuth_kernel()`, `range_kernel()`, `sigma_azimuth()`, `sigma_range()`). (#175)
- `Debug`, `Clone` and `PartialEq` for `ThresholdFilter`, `MedianFilter`, `GaussianSmooth`,
  `CorrelationCoefficientFilter`, `StormRelativeVelocity`, `VerticalCrossSection` and `StormCellDetector`;
  `Debug`, `Clone`, `Copy`, `Default` and `PartialEq` for `CompositeReflectivity`; and `Debug` for `SweepPipeline`.
  (#175)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- `derived::VerticalCrossSection::new` takes `azimuth_degrees: f64`. (#164)
- `ScanDerivedProduct::compute` no longer takes a `&Scan` first argument: it is now
  `compute(&self, fields, coord_system, output_extent, output_resolution)`. Drop the argument at call sites and in
  implementations. (#175)
- `ThresholdFilter`, `MedianFilter` and `GaussianSmooth` have private fields: replace struct literals such as
  `ThresholdFilter { min: Some(5.0), max: None }` with `ThresholdFilter::new(Some(5.0), None)?`, and field reads with
  the getters. (#175)
- Invalid `MedianFilter` kernel sizes and `GaussianSmooth` sigmas are reported by `new` instead of
  `SweepProcessor::process`, and `GaussianSmooth::new` rejects NaN and infinite sigmas, which were previously
  accepted. (#175)
- Angle accessors use the `nexrad-model` stems: `detection::StormCell::elevation_degrees()` →
  `elevation_angle_degrees()` and `max_reflectivity_azimuth_degrees()` → `max_reflectivity_azimuth_angle_degrees()`.
  (#182)
- `derived::VerticalCrossSection::new` parameters are named `azimuth_angle_degrees` and `max_altitude_meters`
  (formerly `azimuth_degrees` and `max_altitude_m`); the types are unchanged. (#182)

### Removed

- The `ScanProcessor` trait, which had no implementations. Use a free function or your own trait. (#175)
