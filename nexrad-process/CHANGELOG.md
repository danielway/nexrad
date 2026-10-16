# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.1`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- Validating constructors `filter::ThresholdFilter::new(min, max)`,
  `filter::MedianFilter::new(azimuth_kernel_radials, range_kernel_gates)` and
  `filter::GaussianSmooth::new(sigma_azimuth_radials, sigma_range_gates)`, each returning
  `Err(Error::InvalidParameter)` for invalid parameters, with getters for every parameter (`min()`, `max()`,
  `azimuth_kernel_radials()`, `range_kernel_gates()`, `sigma_azimuth_radials()`, `sigma_range_gates()`); kernels are
  in radials and gates. (#175, #193)
- `Debug`, `Clone` and `PartialEq` for `ThresholdFilter`, `MedianFilter`, `GaussianSmooth`,
  `CorrelationCoefficientFilter`, `StormRelativeVelocity`, `VerticalCrossSection` and `StormCellDetector`;
  `Debug`, `Clone`, `Copy`, `Default` and `PartialEq` for `CompositeReflectivity`; and `Debug` for `SweepPipeline`.
  (#175)
- `Error::Model(nexrad_model::Error)`, returned when a model constructor rejects an output, such as a
  `CompositeReflectivity::compute` extent outside valid latitudes and longitudes. Its `Display` text is
  `model error`; the model error is its `source()`. (#185, #195)
- `CartesianGrid`, the validated output grid of a `ScanDerivedProduct`: `CartesianGrid::new(extent, width, height)`
  applies the `CartesianField::new` rules, with `extent()`, `width()` and `height()` getters. (#193)
- `SweepPipeline::len()`, `is_empty()` and `steps()`, which iterates the steps as `&dyn SweepProcessor`. (#193)
- Getters for every processor parameter: `VerticalCrossSection::{azimuth_angle_degrees, max_range_km,
  max_altitude_meters, width, height}()`, `StormRelativeVelocity::{storm_direction_degrees, storm_speed_mps}()`,
  `StormCellDetector::{reflectivity_threshold_dbz, min_gate_count}()` and
  `CorrelationCoefficientFilter::{threshold, correlation_coefficient}()`. (#193)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- `ScanDerivedProduct::compute(&self, scan, fields, coord_system, output_extent, output_resolution)` →
  `compute(&self, fields, coordinate_system, grid: &CartesianGrid)`: the `&Scan` argument is dropped and a
  `CartesianGrid` replaces the `&GeoExtent` and `(usize, usize)` resolution, at call sites and in implementations
  (also `CompositeReflectivity::compute`). (#175, #193)
- `ThresholdFilter`, `MedianFilter` and `GaussianSmooth` have private fields: replace struct literals such as
  `ThresholdFilter { min: Some(5.0), max: None }` with `ThresholdFilter::new(Some(5.0), None)?`, and field reads with
  the getters. (#175)
- Invalid `MedianFilter` kernel sizes and `GaussianSmooth` sigmas are reported by `new` instead of
  `SweepProcessor::process`, and `GaussianSmooth::new` rejects NaN and infinite sigmas, which were previously
  accepted. (#175)
- Angle accessors use the `nexrad-model` stems: `detection::StormCell::elevation_degrees()` →
  `elevation_angle_degrees()` and `max_reflectivity_azimuth_degrees()` → `max_reflectivity_azimuth_angle_degrees()`.
  (#182)
- `ThresholdFilter` and `CorrelationCoefficientFilter` mark the gates they remove `GateStatus::Filtered` and keep their
  values, instead of setting `GateStatus::NoData` with `0.0`. The other processors use only `Valid` gates, so they
  pass `Filtered` gates through unchanged (smoothing, storm-relative velocity) or skip them (composite reflectivity,
  vertical cross-section, storm cell detection). (#187)
- `derived::VerticalCrossSection::compute(&self, fields, coordinate_system)` takes the radar's
  `RadarCoordinateSystem` and places each tilt with its `ground_range_km` and `beam_height_meters`: the horizontal axis
  is ground range and the vertical axis is altitude above mean sea level, including the antenna height. (#188)
- `SweepProcessor` and `ScanDerivedProduct` require `Send + Sync`, so `SweepPipeline` and boxed processors can be
  shared across threads. (#193)
- `derived::VerticalCrossSection::new(azimuth_degrees: f32, max_range_km, max_altitude_m, width, height)` is replaced
  by `VerticalCrossSection::builder(azimuth_angle_degrees: f64)` with `max_range_km`, `max_altitude_meters`, `width`
  and `height` setters (defaulting to 230 km, 18 000 m, 600 and 300) and a validating `build()`. (#164, #182, #193)
- `Error` derives `Clone` and `PartialEq` and carries structured data: `InvalidParameter`, `InvalidGeometry` and
  `MissingData` are struct variants with `&'static str` fields instead of `String` payloads, and the new
  `GeometryMismatch { parameter }` and `UnexpectedProduct { expected, actual }` report mismatched inputs. (#193)
- Every floating-point processor parameter is checked finite: `StormRelativeVelocity::new` rejects a non-finite
  speed or a direction outside `0.0..360.0`, `StormCellDetector::new` a non-finite threshold, and
  `CorrelationCoefficientFilter::new` a threshold that is NaN or outside `0.0..=1.0` (`0.0` is now accepted).
  (#193)
- `CompositeReflectivity::compute` requires every field to hold `Product::Reflectivity` and labels its output
  "Composite Reflectivity" in "dBZ". `CorrelationCoefficientFilter::new` requires a
  `Product::CorrelationCoefficient` field, and `StormRelativeVelocity` a `Product::Velocity` input; its output is
  labelled "Storm-Relative Velocity" in "m/s" with no product. Mismatches return `Error::UnexpectedProduct`.
  (#193)
- `derived::VerticalCrossSection::compute` and `detection::StormCellDetector::detect` require every input field to
  hold `Product::Reflectivity`, returning `Error::UnexpectedProduct` otherwise. (#193)
- `detection::StormCell::bounds()` returns a `nexrad_model::geo::GeoExtent` by value. (#193)

### Removed

- The `ScanProcessor` trait, which had no implementations. Use a free function or your own trait. (#175)
- `detection::StormCellBounds`, which duplicated `nexrad_model::geo::GeoExtent`: read
  `bounds().min.latitude` instead of `bounds().min_latitude()`, and so on. (#193)
- The `pipeline` module path: `nexrad_process::pipeline::SweepPipeline` is `nexrad_process::SweepPipeline`. (#193)

### Fixed

- `CompositeReflectivity` and `VerticalCrossSection` no longer sample a sector or partial sweep at the far end of its
  azimuths across north, through the `SweepField::value_at_polar` fix. (#185)
- `CorrelationCoefficientFilter` compares the input and correlation coefficient fields by azimuth angles, first gate
  range and gate interval, returning `Error::GeometryMismatch` when they differ, instead of only the azimuth count,
  so mis-registered gates are never masked by index. Gate counts may still differ. (#193)
- `StormRelativeVelocity` output is no longer labelled as the input velocity moment. (#193)
- `StormCellDetector` weights the centroid by linear reflectivity factor (`10^(dBZ/10)`) instead of raw dBZ, which
  is zero or negative at and below 0 dBZ and gave a NaN or misplaced centroid for low thresholds. (#193)
- `MedianFilter` and `GaussianSmooth` wrap azimuth across north only when the field covers the full circle (the rule
  `SweepField::value_at_polar` uses); a sector's edges are boundaries instead of neighbours. (#193)
- `StormCellDetector` joins gates across north only when the field covers the full circle, so a sector's two edges
  are no longer merged into one cell. (#193)
