# NEXRAD Process

[![Crates.io](https://img.shields.io/crates/v/nexrad-process)](https://crates.io/crates/nexrad-process)
[![Docs.rs](https://docs.rs/nexrad-process/badge.svg)](https://docs.rs/nexrad-process)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Processing algorithms for NEXRAD weather radar data.

## Processing Traits

Two traits cover different processing scopes:

| Trait | Scope | Input → Output |
|-------|-------|---------------|
| `SweepProcessor` | Single sweep | `SweepField → SweepField` |
| `ScanDerivedProduct` | Full scan | `[SweepField] → CartesianField` |

## Filters

Built-in `SweepProcessor` implementations:

| Filter | Effect |
|--------|--------|
| `ThresholdFilter` | Mask gates outside a min/max value range |
| `GaussianSmooth` | Gaussian blur with configurable azimuth/range sigma |
| `MedianFilter` | Non-linear noise reduction with configurable kernel size |
| `CorrelationCoefficientFilter` | Quality-based noise removal using CC data |

## Derived Products

| Product | Trait | Output |
|---------|-------|--------|
| `CompositeReflectivity` | `ScanDerivedProduct` | `CartesianField` — max reflectivity across all tilts |
| `StormRelativeVelocity` | `SweepProcessor` | `SweepField` — velocity with storm motion removed |
| `VerticalCrossSection` | (standalone) | `VerticalField` — RHI-style range-height display |

## Storm Cell Detection

`detection::StormCellDetector` finds contiguous regions of reflectivity above a threshold in a `SweepField` and reports
each as a `StormCell` with its centroid, bounds, area and reflectivity statistics.

## Constructing Processors

Every processor with parameters is built with a validating `new` that returns `Err(Error::InvalidParameter)` for
invalid input (a NaN bound, an even kernel size, a non-positive sigma, ...), so invalid configuration is reported at
construction rather than by `process`. Parameters are read back with getters such as `ThresholdFilter::min()`.

## Pipeline

Processors can be composed into a `SweepPipeline`:

```rust,ignore
use nexrad_process::filter::{GaussianSmooth, ThresholdFilter};
use nexrad_process::SweepPipeline;

let filtered = SweepPipeline::new()
    .then(ThresholdFilter::new(Some(5.0), None)?)
    .then(GaussianSmooth::new(1.0, 1.0)?)
    .execute(&field)?;
```

The pipeline itself implements `SweepProcessor`, so pipelines can be nested.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.

## Crate Boundaries

This crate provides **processing algorithms** only:

- No I/O operations (operates on `nexrad-model` field types)
- No rendering (produces field data, not images)

For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
