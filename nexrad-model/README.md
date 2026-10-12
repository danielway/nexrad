# NEXRAD Model

[![Crates.io](https://img.shields.io/crates/v/nexrad-model)](https://crates.io/crates/nexrad-model)
[![Docs.rs](https://docs.rs/nexrad-model/badge.svg)](https://docs.rs/nexrad-model)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

A common model for representing NEXRAD weather radar data. Provides an ergonomic API which is documented for an audience
who is not necessarily familiar with the NOAA Archive II format.

## Data Model

Radar data is organized into a hierarchical structure:

```
Scan                           A complete radar volume
├── VolumeCoveragePattern      Scanning strategy (elevation cuts, waveforms)
├── Site                       Radar site metadata (ICAO ID, location, elevation)
└── Vec<Sweep>                 One sweep per rotation at a fixed elevation angle
    └── Vec<Radial>            One radial per beam direction
        ├── reflectivity()     → Option<&MomentData>
        ├── velocity()         → Option<&MomentData>
        ├── spectrum_width()   → Option<&MomentData>
        ├── differential_reflectivity()
        ├── differential_phase()
        ├── correlation_coefficient()
        ├── clutter_filter_power()
        └── moment(Product)    → Option<&MomentData>
```

Every moment, including clutter filter power, is a `MomentData` that knows its `Product`.
`Radial::moment(product)` is the uniform accessor; the named getters are shorthands for it.
`MomentData` exposes the raw encoding (`scale()`, `offset()`, `data_word_size()`,
`raw_values()`, `raw_gate_values()`) and decodes gates through `iter()`/`values()`, which
yield `MomentValue` variants:

- `MomentValue::Value(f32)` — a valid measurement
- `MomentValue::BelowThreshold` — signal below the signal-to-noise threshold
- `MomentValue::RangeFolded` — ambiguous range due to Doppler aliasing
- `MomentValue::CfpStatus(CfpStatus)` — a clutter filter power status code (raw values 0–7;
  only produced for `Product::ClutterFilterPower`)

## Field Types

For processing and rendering, moment data is extracted into flat 2D grids:

| Type | Axes | Use Case |
|------|------|----------|
| `SweepField` | azimuth × range (polar) | Single-sweep PPI display, filtering |
| `CartesianField` | lat × lon (geographic) | Composite reflectivity, echo tops |
| `VerticalField` | range × altitude | RHI / vertical cross-sections |

Each field stores parallel `f32` and `GateStatus` arrays in row-major order for efficient numerical processing.

`Sweep::field(product)` extracts a `SweepField` from one sweep; `Scan::fields(product)` and
`Scan::first_field(product)` do the same across a scan's sweeps.

## Geographic Types

The `geo` module provides coordinate types and transformations:

- `GeoPoint` / `GeoPoint3D` — WGS-84 latitude/longitude (with optional altitude)
- `GeoExtent` — axis-aligned bounding box
- `PolarPoint` — azimuth, range, and elevation from a radar
- `RadarCoordinateSystem` — converts between polar and geographic coordinates

`Scan::coordinate_system()` builds a `RadarCoordinateSystem` from a scan's site metadata.

## Site Registry

A compile-time registry of all operational NEXRAD WSR-88D radar sites is available through the `meta::registry` module:

```rust
use nexrad_model::meta::registry;

// Look up a site
let site = registry::site_by_id("KTLX").unwrap();
println!("{}: {}, {}", site.id, site.city, site.state);

// Find the nearest site to a location
let nearest = registry::nearest_site(35.4676, -97.5164).unwrap();
```

## Features

- `uom`: Use the `uom` crate for type-safe units of measure. Enables methods returning `Length`, `Velocity`, etc. instead of raw `f64` values.
- `serde`: Implement `serde::Serialize` and `serde::Deserialize` for all model types.
- `chrono`: Use the `chrono` crate for date and time types. Enables `collection_time()` returning `DateTime<Utc>` instead of raw timestamps.

## Crate Boundaries

This crate is a **pure data model**:

- No I/O operations (file, network, stdio)
- No binary parsing or encoding
- No rendering or visualization

All I/O, parsing, and rendering concerns are handled by separate crates in the NEXRAD library suite. For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
