# nexrad-model

[![Crates.io](https://img.shields.io/crates/v/nexrad-model)](https://crates.io/crates/nexrad-model)
[![Docs.rs](https://docs.rs/nexrad-model/badge.svg)](https://docs.rs/nexrad-model)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/nexrad-model)](https://github.com/danielway/nexrad#installation)
[![License](https://img.shields.io/crates/l/nexrad-model)](https://github.com/danielway/nexrad/blob/main/LICENSE)

Types for NEXRAD weather radar data: scans, sweeps, radials and their moments, documented for readers who don't know
the Archive II format.

One of the [nexrad crates](https://github.com/danielway/nexrad). Most programs depend on [`nexrad`](https://crates.io/crates/nexrad), which re-exports this crate.

## Scope

The nexrad crates read NEXRAD Level II data in the Archive II format (ICD 2620010) and the RDA/RPG messages it carries
(ICD 2620002). WSR-88D volumes decode, and so do TDWR Level II volumes. The intended scope for 1.0 is the whole archive:
volumes with legacy message 1 radials, volumes from before 2008 that carry no VCP message, and gzip-compressed volumes
back to 1991 are all inputs. `_MDM` objects (Model Data, message type 29) and archive tarballs are not read, and Level
III products are out of scope.

## Data Model

Radar data is organized into a hierarchical structure:

```text
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
- `MomentValue::RangeFolded` — the gate is flagged range folded (its echo came from beyond the maximum unambiguous range)
- `MomentValue::CfpStatus(CfpStatus)` — a clutter filter power status code (raw values 0–7;
  only produced for `Product::ClutterFilterPower`)

## Field Types

For processing and rendering, moment data is extracted into flat 2D grids:

| Type | Axes | Use Case |
|------|------|----------|
| `SweepField` | azimuth × range (polar) | Single-sweep PPI display, filtering |
| `CartesianField` | lat × lon (geographic) | Composite reflectivity |
| `VerticalField` | range × altitude | RHI / vertical cross-sections |

Each field stores parallel `f32` and `GateStatus` arrays in row-major order for efficient numerical processing.

`Sweep::field(product)` extracts a `SweepField` from one sweep; `Scan::fields(product)` and
`Scan::first_field(product)` do the same across a scan's sweeps.

## Constructing Model Types

Types with invariants have private fields and are built with builders: `Radial::builder`, `MomentData::builder`,
`ElevationCut::builder`, `VolumeCoveragePattern::builder`, `SweepField::builder` and `Site::builder`. Each takes the required values up front
and has a setter per optional field:

```rust
use nexrad_model::data::{Product, SweepField};
use nexrad_model::meta::Site;

fn main() -> nexrad_model::Result<()> {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .build()?;

    let field = SweepField::builder("Reflectivity", "dBZ")
        .product(Product::Reflectivity)
        .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
        .gate_interval_km(0.25)
        .gate_count(1832)
        .build()?;

    println!("{} at {}: {} gates", field.label(), site.identifier(), field.gate_count());
    Ok(())
}
```

Constructors and builders with invariants (such as `SiteBuilder::build`, `SweepFieldBuilder::build`,
`RadialBuilder::build`, `MomentDataBuilder::build` and `ElevationCutBuilder::build`) validate their input and return `nexrad_model::Result`;
field indexing (`get`, the row accessors) returns `Option`. `Error` and `Result` are
re-exported at the crate root.

## Geographic Types

The `geo` module provides coordinate types and transformations:

- `GeoPoint` / `GeoPoint3D` — latitude/longitude (with optional altitude above mean sea level)
- `GeoExtent` — axis-aligned bounding box
- `PolarPoint` — azimuth, range, and elevation from a radar
- `RadarCoordinateSystem` — converts between polar and geographic coordinates and gives beam height and ground range,
  using a spherical earth of radius 6371 km with 4/3 effective-radius refraction and the antenna height above mean sea
  level

`Scan::coordinate_system()` builds a `RadarCoordinateSystem` from a scan's site metadata.

## Site Registry

A compile-time registry of all operational NEXRAD WSR-88D radar sites is available through the `meta::registry` module:

```rust
use nexrad_model::meta::registry;

// Look up a site
let site = registry::site_by_id("KTLX").unwrap();
println!("{site}"); // KTLX (Oklahoma City, OK)

// Find the nearest site to a location
let nearest = registry::nearest_site(35.4676, -97.5164).unwrap();
```

## Features

- `serde`: Implement `serde::Serialize` and `serde::Deserialize` for the data, geometry and site types (not for the builders, `SiteEntry` or `Error`).
- `chrono`: Use the `chrono` crate for date and time types. Enables `collection_time()` returning `DateTime<Utc>` instead of raw timestamps.

Accessors return plain numbers with the unit in the name (`elevation_angle_degrees()`, `gate_interval_km()`).
Unit-typed accessors (for example with the `uom` crate) may be offered later as a separate crate or extension trait.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.
To upgrade from 0.x or a release candidate, see [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).

## Crate Boundaries

This crate is a **pure data model**:

- No I/O operations (file, network, stdio)
- No binary parsing or encoding
- No rendering or visualization

All I/O, parsing, and rendering concerns are handled by separate crates in the NEXRAD library suite. For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
