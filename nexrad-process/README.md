# nexrad-process

[![Crates.io](https://img.shields.io/crates/v/nexrad-process)](https://crates.io/crates/nexrad-process)
[![Docs.rs](https://docs.rs/nexrad-process/badge.svg)](https://docs.rs/nexrad-process)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/nexrad-process)](https://github.com/danielway/nexrad#installation)
[![License](https://img.shields.io/crates/l/nexrad-process)](https://github.com/danielway/nexrad/blob/main/LICENSE)

Filters and derived products for NEXRAD weather radar sweeps.

![Reflectivity before and after filtering](https://raw.githubusercontent.com/danielway/nexrad/nexrad-process@1.0.0/docs/images/processing.png)

*Left: lowest-tilt reflectivity from KDMX (Des Moines, Iowa), 5 March 2022 23:23 UTC. Right: the same sweep after `CorrelationCoefficientFilter` removes gates with a correlation coefficient below 0.8 and `MedianFilter` applies a 3×3 median.*

One of the [nexrad crates](https://github.com/danielway/nexrad). Most programs depend on [`nexrad`](https://crates.io/crates/nexrad), which re-exports this crate.

## Scope

The [repository README](https://github.com/danielway/nexrad#scope) describes which Level II data the nexrad crates
read.

## Processing Trait

`SweepProcessor` transforms one sweep (`SweepField → SweepField`); `SweepPipeline` chains processors. Products that
combine the sweeps of a full scan have their own inherent `compute` methods.

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
| `CompositeReflectivity` | (standalone) | `CartesianField` — max reflectivity across all tilts |
| `StormRelativeVelocity` | `SweepProcessor` | `SweepField` — velocity with storm motion removed |
| `VerticalCrossSection` | (standalone) | `VerticalField` — RHI-style range-height display |

## Threshold Region Detection

`detection::ThresholdRegionDetector` finds connected regions of gates at or above one reflectivity threshold in a
`SweepField` and reports each as a `ThresholdRegion` with its centroid, bounds, area and reflectivity statistics. The
names describe the computation; a region is not a meteorological classification.

## Constructing Processors

Every processor with parameters is built with a validating `new` that returns `Err(Error::InvalidParameter)` for
invalid input (a NaN bound, an even kernel size, a non-positive sigma, ...), so invalid configuration is reported at
construction rather than by `process`. Parameters are read back with getters such as `ThresholdFilter::min()`.

## Pipeline

Processors can be composed into a `SweepPipeline`:

```rust
use nexrad_model::data::SweepField;
use nexrad_process::filter::{GaussianSmooth, ThresholdFilter};
use nexrad_process::SweepPipeline;

fn smooth(field: &SweepField) -> nexrad_process::Result<SweepField> {
    SweepPipeline::new()
        .then(ThresholdFilter::new(Some(5.0), None)?)
        .then(GaussianSmooth::new(1.0, 1.0)?)
        .execute(field)
}
```

The pipeline itself implements `SweepProcessor`, so pipelines can be nested.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.
To upgrade from 0.x or a release candidate, see [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).

## Crate Boundaries

This crate provides **processing algorithms** only:

- No I/O operations (operates on `nexrad-model` field types)
- No rendering (produces field data, not images)

For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
