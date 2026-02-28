# NEXRAD Process

[![Crates.io](https://img.shields.io/crates/v/nexrad-process)](https://crates.io/crates/nexrad-process)
[![Docs.rs](https://docs.rs/nexrad-process/badge.svg)](https://docs.rs/nexrad-process)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Processing algorithms for NEXRAD weather radar data.

## Processing Traits

Three traits cover different processing scopes:

| Trait | Scope | Input → Output |
|-------|-------|---------------|
| `SweepProcessor` | Single sweep | `SweepField → SweepField` |
| `ScanProcessor` | Multiple sweeps | `[SweepField] → [SweepField]` |
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

## Pipeline

Processors can be composed into a `SweepPipeline`:

```rust,ignore
use nexrad_process::{SweepPipeline, filter::ThresholdFilter, filter::GaussianSmooth};

let filtered = SweepPipeline::new()
    .then(ThresholdFilter { min: Some(5.0), max: None })
    .then(GaussianSmooth { sigma_azimuth: 1.0, sigma_range: 1.0 })
    .execute(&field)?;
```

The pipeline itself implements `SweepProcessor`, so pipelines can be nested.

## Crate Boundaries

This crate provides **processing algorithms** only:

- No I/O operations (operates on `nexrad-model` field types)
- No rendering (produces field data, not images)

For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
