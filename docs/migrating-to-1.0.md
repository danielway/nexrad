# Migrating to 1.0

This guide takes code written against a `1.0.0-rc.N` release candidate or a 0.x release of the NEXRAD crates and gets it
compiling and behaving correctly on 1.0.

## Contents

- [Introduction](#introduction)
  - [Versions covered](#versions-covered)
  - [How to use this guide](#how-to-use-this-guide)
  - [Changes that apply to every crate](#changes-that-apply-to-every-crate)
- [Upgrading from a release candidate](#upgrading-from-a-release-candidate)
  - [nexrad-model from 1.0.0-rc.2](#nexrad-model-from-100-rc2)
  - [nexrad-decode from 1.0.0-rc.3](#nexrad-decode-from-100-rc3)
  - [nexrad-data from 1.0.0-rc.7](#nexrad-data-from-100-rc7)
  - [nexrad-process from 1.0.0-rc.1](#nexrad-process-from-100-rc1)
  - [nexrad-render from 1.0.0-rc.4](#nexrad-render-from-100-rc4)
  - [nexrad from 1.0.0-rc.4](#nexrad-from-100-rc4)
  - [Serialized data](#serialized-data)
- [Upgrading from 0.x](#upgrading-from-0x)
  - [The crate split](#the-crate-split)
  - [nexrad 0.0.3](#nexrad-003)
  - [nexrad-model 0.1.0](#nexrad-model-010)
  - [nexrad-decode 0.1.2](#nexrad-decode-012)
  - [nexrad-data 0.2.0](#nexrad-data-020)
  - [Conceptual changes since 0.x](#conceptual-changes-since-0x)
- [Worked examples](#worked-examples)
  - [Load a local Archive II file](#load-a-local-archive-ii-file)
  - [List and download from AWS](#list-and-download-from-aws)
  - [Render a reflectivity sweep to PNG](#render-a-reflectivity-sweep-to-png)
- [Appendix: complete change tables](#appendix-complete-change-tables)
  - [nexrad-model 1.0.0-rc.2 to 1.0](#nexrad-model-100-rc2-to-10)
  - [nexrad-decode 1.0.0-rc.3 to 1.0](#nexrad-decode-100-rc3-to-10)
  - [nexrad-data 1.0.0-rc.7 to 1.0](#nexrad-data-100-rc7-to-10)
  - [nexrad-process 1.0.0-rc.1 to 1.0](#nexrad-process-100-rc1-to-10)
  - [nexrad-render 1.0.0-rc.4 to 1.0](#nexrad-render-100-rc4-to-10)
  - [nexrad 1.0.0-rc.4 to 1.0](#nexrad-100-rc4-to-10)
  - [0.x baselines](#0x-baselines)
  - [How these tables were produced](#how-these-tables-were-produced)

## Introduction

### Versions covered

The six library crates release 1.0.0 together. This guide covers upgrades from these versions:

| Crate | Last release candidate | Last 0.x release |
|---|---|---|
| `nexrad` | `1.0.0-rc.4` | `0.0.3` |
| `nexrad-model` | `1.0.0-rc.2` | `0.1.0` |
| `nexrad-decode` | `1.0.0-rc.3` | `0.1.2` |
| `nexrad-data` | `1.0.0-rc.7` | `0.2.0` |
| `nexrad-process` | `1.0.0-rc.1` | none |
| `nexrad-render` | `1.0.0-rc.4` | none |

If you are on an earlier release candidate, follow the release-candidate section; the extra changes between your
release candidate and the last one are small and the compiler will point to them. `nexrad 0.1.0-rc1` contained only an
`Error` enum and is not covered.

The minimum supported Rust version is **1.88** for every crate. The release candidates declared 1.75, which was already
incorrect: they used items that entered the prelude in 1.80, and their dependencies needed 1.88.

### How to use this guide

The `nexrad` crate is a facade. It re-exports the five sub-crates as `nexrad::model`, `nexrad::decode`, `nexrad::data`,
`nexrad::process` and `nexrad::render`, and adds the I/O entry points `load`, `load_file`, `download`,
`download_latest`, `download_at`, `list_scans` and `stream` plus a unified `nexrad::Error`.

- **If you use the facade**, read the facade section for your baseline, then the sections of the sub-crates whose types
  you touch. Most facade users touch `nexrad::model` (scans, sweeps, radials) and often `nexrad::render`.
- **If you depend on sub-crates directly**, read the section for each one. They are in dependency order: `nexrad-model`
  first, `nexrad` last.
- **If you are on 0.x**, read [Upgrading from 0.x](#upgrading-from-0x) first, then the release-candidate sections for
  the crates you use. The 0.x section maps 0.x items to their 1.0 names and does not repeat the release-candidate
  tables.

The per-crate CHANGELOGs ([`nexrad`](../nexrad/CHANGELOG.md), [`nexrad-model`](../nexrad-model/CHANGELOG.md),
[`nexrad-decode`](../nexrad-decode/CHANGELOG.md), [`nexrad-data`](../nexrad-data/CHANGELOG.md),
[`nexrad-process`](../nexrad-process/CHANGELOG.md), [`nexrad-render`](../nexrad-render/CHANGELOG.md)) list every change
with the pull request that made it. [architecture.md](architecture.md) states the rules that shaped 1.0: the
[layering rules](architecture.md#layering-rules), [feature architecture](architecture.md#feature-architecture),
[error architecture](architecture.md#error-architecture), [naming conventions](architecture.md#naming-conventions) and
[enum stability](architecture.md#enum-stability).

The 1.0 snippets in this guide are written against the facade (`nexrad::model::...`). If you depend on a sub-crate
directly, replace `nexrad::model` with `nexrad_model`, `nexrad::data` with `nexrad_data`, and so on; the paths below the
crate root are identical.

### Changes that apply to every crate

These four changes touch every crate. The per-crate sections below assume you have read them.

**1. `Error` and `Result` at every crate root.** Each crate re-exports its error types at the root:
`nexrad_model::Error`, `nexrad_decode::Error`, `nexrad_data::Error` (plus `nexrad_data::AwsError` with `aws`),
`nexrad_process::Error`, `nexrad_render::Error` and `nexrad::Error`, each with a matching `Result`. The `result` modules
still exist, so `nexrad_data::result::Error` keeps working. Nothing to change, but the short paths are available.

**2. Every `Error` enum is `#[non_exhaustive]`.** So are `nexrad_data::AwsError`, `nexrad_model::data::Product`,
`nexrad_model::data::VcpNumber`, `nexrad_decode::messages::{MessageContents, MessageType, RedundantChannel, VcpNumber}`,
`nexrad_decode::messages::digital_radar_data::DataBlockName` and `nexrad_render::Projection`. A `match` on any of them
outside its crate needs a wildcard arm:

```rust
// 1.0.0-rc.N: compiled without a wildcard
match product {
    Product::Reflectivity => "dBZ",
    Product::Velocity | Product::SpectrumWidth => "m/s",
    Product::DifferentialReflectivity => "dB",
    Product::DifferentialPhase => "deg",
    Product::CorrelationCoefficient => "",
    Product::ClutterFilterPower => "dB",
}

// 1.0
match product {
    Product::Reflectivity => "dBZ",
    // ...
    _ => product.unit(),
}
```

The deliberately exhaustive enums (`MomentValue`, `GateStatus`, `RadialStatus`, `WaveformType`, ...) are listed under
[Enum Stability](architecture.md#enum-stability).

**3. Acronyms are written as words.** Type and variant names follow Rust's RFC 430 casing: `VCPNumber` → `VcpNumber`,
`RDAStatus` → `RdaStatus`, `AWSError` → `AwsError`, `CFPStatus` → `CfpStatus`, `SZ2Phase` → `Sz2Phase`. The terse
waveform variants got descriptive names (`WaveformType::CS` → `ContiguousSurveillance`). Each crate section lists its
renames, and the [appendix](#appendix-complete-change-tables) has the complete tables. If you persisted model data with
serde, read [Serialized data](#serialized-data): these renames change the serialized form.

**4. Optional dependencies are no longer features.** Every crate uses `dep:` syntax, so names such as `nexrad-model`,
`tokio`, `futures`, `reqwest`, `rayon`, `xml` and `once_cell` are no longer feature names. Features are named for what
they enable: `model`, `aws`, `parallel`, `uom`, `serde`, `chrono`. No crate depends on an async runtime any more; see
[No Async Runtime Dependency](architecture.md#no-async-runtime-dependency).

## Upgrading from a release candidate

### nexrad-model from 1.0.0-rc.2

#### Feature flags

No feature changes. `serde`, `chrono` and `uom` are still opt-in.

#### Error handling

- `nexrad_model::Error` is `#[non_exhaustive]`: add a wildcard arm.
- Its variants carry structured data (#185). `ElevationMismatch` is a struct variant: match it as
  `Error::ElevationMismatch { expected, actual }` or `Error::ElevationMismatch { .. }`. New variants, each naming the
  offending argument in a `parameter: &'static str` field:
  - `InvalidGeometry { parameter, reason }`, from `SweepFieldBuilder::build`, `RadialBuilder::build`,
    `CartesianField::new` and `VerticalField::new`;
  - `InvalidMomentData { parameter, reason }`, from `MomentData::from_fixed_point` for an unsupported
    `data_word_size`, a `gate_count` above `u16::MAX`, or a non-finite `scale` or `offset`;
  - `MomentDataLength { expected, actual }`, from `MomentData::from_fixed_point` when `values` does not hold exactly
    `gate_count` gates (lengths in bytes);
  - `InvalidSite { parameter, reason }`, from `SiteBuilder::build` and `registry::SiteEntry::to_site`;
  - `IndexOutOfBounds { row, column, rows, columns }`, from the field `set` methods.
- `Error` implements `Clone` and `PartialEq`.

#### Removed and renamed items

Replace constructors with builders. Each builder takes the required values as arguments, has a setter per optional
value, and ends with `.build()`:

```rust
// 1.0.0-rc.2
let radial = Radial::new(
    timestamp, azimuth_number, azimuth_angle_degrees, 0.5, RadialStatus::IntermediateRadialData,
    elevation_number, elevation_angle_degrees, Some(reflectivity), None, None, None, None, None, None,
);
let site = Site::new(*b"KDMX", 41.7312, -93.7229, 299, 34);

// 1.0
let radial = Radial::builder(azimuth_number, azimuth_angle_degrees, elevation_number, elevation_angle_degrees)
    .collection_timestamp_millis(timestamp)
    .azimuth_spacing_degrees(0.5)
    .moment(reflectivity) // stored under reflectivity.product()
    .build()?; // validates the angles
let site = Site::builder(*b"KDMX", 41.7312, -93.7229)
    .site_elevation_meters(299)
    .feedhorn_height_meters(34)
    .build()?; // validates the identifier and location
```

Constructors that have invariants to check return `Result`, and field indexing returns `Option` ({PR}):

```rust
// 1.0.0-rc.2
let moment = MomentData::from_fixed_point(Product::Reflectivity, 1832, 2125, 250, 8, 2.0, 66.0, bytes);
let sweep = Sweep::new(1, radials);
let mut grid = CartesianField::new("Composite", "dBZ", extent, 800, 800);
grid.set(row, col, dbz, GateStatus::Valid);
let (value, status) = field.get(azimuth_idx, gate_idx);
let row = field.radial_values(azimuth_idx);

// 1.0
let moment = MomentData::from_fixed_point(Product::Reflectivity, 1832, 2125, 250, 8, 2.0, 66.0, bytes)?;
let sweep = Sweep::new(1, radials)?;
let mut grid = CartesianField::new("Composite", "dBZ", extent, 800, 800)?;
grid.set(row, col, dbz, GateStatus::Valid)?;
let Some((value, status)) = field.get(azimuth_idx, gate_idx) else { return Ok(()) };
let Some(row) = field.radial_values(azimuth_idx) else { return Ok(()) };
```

| 1.0.0-rc.2 item | 1.0 replacement | Notes / PR |
|---|---|---|
| `Radial::new(..)` (14 arguments) | `Radial::builder(azimuth_number, azimuth_angle_degrees, elevation_number, elevation_angle_degrees)` with `.collection_timestamp_millis`, `.azimuth_spacing_degrees`, `.radial_status`, `.moment(data)`, then `.build()?` | Defaults: timestamp `0`, spacing `1.0`, status `IntermediateRadialData`. `build()` returns `Result`. #165, #170, #185 |
| `ElevationCut::new(..)` (22 arguments) | `ElevationCut::builder(elevation_angle_degrees, waveform_type, channel_configuration)` | **Argument order changed**: waveform now comes before channel configuration. #165 |
| `VolumeCoveragePattern::new(pattern_number: u16, ..)` | `VolumeCoveragePattern::builder(VcpNumber::from_number(n))` | #165 |
| `Site::new(id, lat, lon, height, tower_height)` | `Site::builder(id, lat, lon).site_elevation_meters(..).feedhorn_height_meters(..).build()?` | Latitude and longitude are `f64`; `build()` returns `Result`. #164, #165, #182, #185 |
| `SweepField::new_empty(label, unit, ..)` | `SweepField::builder(label, unit)` with setters, `.build()?` | `build()` returns `Result` and validates the geometry. #165 |
| `VCPNumber` | `VcpNumber` | #160 |
| `CFPStatus` | `CfpStatus` | #160 |
| `ChannelConfiguration::SZ2Phase` | `ChannelConfiguration::Sz2Phase` | #160 |
| `WaveformType::{CS, CDW, CDWO, B, SPP}` | `ContiguousSurveillance`, `ContiguousDopplerWithAmbiguityResolution`, `ContiguousDopplerWithoutAmbiguityResolution`, `Batch`, `StaggeredPulsePair` | #160 |
| `RadialStatus::ElevationStartVCPFinal` | `RadialStatus::ElevationStartVcpFinal` | #160 |
| `DataMoment` trait | Inherent methods on `MomentData`, same names and signatures | Remove `use ...::DataMoment`. #170 |
| `CFPMomentData` | `MomentData` with `Product::ClutterFilterPower` | `Radial::clutter_filter_power()` returns `Option<&MomentData>`. #170 |
| `CFPMomentValue::Status(s)` / `::Value(v)` | `MomentValue::CfpStatus(s)` / `MomentValue::Value(v)` | #170 |
| `MomentData::from_fixed_point(gate_count: u16, ..) -> MomentData` | `MomentData::from_fixed_point(product, gate_count: usize, ..)?` | New first parameter (#170); returns `Result` and `gate_count` is `usize` (#185). |
| `MomentData::gate_count() -> u16` | `MomentData::gate_count() -> usize` | #185 |
| `Sweep::new(elevation_number, radials) -> Sweep` | `Sweep::new(elevation_number, radials)?` | Fails if a radial has another elevation number. #185 |
| `SweepField::get(az, gate) -> (f32, GateStatus)` | `SweepField::get(az, gate) -> Option<(f32, GateStatus)>` | Same for `CartesianField::get` and `VerticalField::get`. #185 |
| `SweepField::set(az, gate, value, status)` | `SweepField::set(az, gate, value, status)?` | Returns `Result<()>`; same for `CartesianField::set` and `VerticalField::set`. #185 |
| `SweepField::{radial_values, radial_values_mut, radial_statuses}(az) -> &[_]` | `-> Option<&[_]>` / `Option<&mut [_]>` | #185 |
| `CartesianField::new(..) -> CartesianField`, `VerticalField::new(..) -> VerticalField` | `-> Result<Self>` | #185 |
| `Site::identifier() -> &[u8; 4]`, `Site::identifier_string() -> String` | `Site::identifier() -> &str` | #185 |
| `registry::SiteEntry::to_site() -> Site` | `to_site() -> Result<Site>` | Every registry entry is valid. #185 |
| `Product::moment_data(&radial)` / `Product::cfp_moment_data(&radial)` | `radial.moment(product)` | #170 |
| `BinaryData` | Not public | It appeared in no public signature. #162 |
| (facade) `nexrad::extract_field(sweep, p)` and friends | `sweep.field(p)`, `scan.fields(p)`, `scan.first_field(p)`, `scan.coordinate_system()` | New model methods. #172 |
| `Site::height_meters()` / `height()` | `Site::site_elevation_meters()` / `site_elevation()` | Above mean sea level. #182 |
| `Site::tower_height_meters()` / `tower_height()` | `Site::feedhorn_height_meters()` / `feedhorn_height()` | Above ground level. #182 |
| `registry::SiteEntry::elevation_meters` | `registry::SiteEntry::site_elevation_meters` | Above mean sea level. `SiteEntry` is `#[non_exhaustive]`, so it cannot be built with a struct literal outside the crate. #182 |
| `Radial::collection_timestamp()` | `Radial::collection_timestamp_millis()` | Milliseconds since the Unix epoch, UTC. #182 |
| `SweepField::elevation_degrees()` | `SweepField::elevation_angle_degrees()` | #182 |
| `SweepField::azimuths()` | `SweepField::azimuth_angles_degrees()` | #182 |
| `SweepField::radial_status_mut(usize)` | `SweepField::radial_statuses_mut(usize)` | Returns `Option<&mut [GateStatus]>`. #182, #185 |
| `VolumeCoveragePattern::doppler_velocity_resolution()` | `VolumeCoveragePattern::doppler_velocity_resolution_mps()` | #182 |
| `VolumeCoveragePattern::{sails_cuts, mrle_cuts}()` | `VolumeCoveragePattern::{sails_cut_count, mrle_cut_count}()` | #182 |
| `geo::PolarPoint::{azimuth_degrees, elevation_degrees}` | `geo::PolarPoint::{azimuth_angle_degrees, elevation_angle_degrees}` | #182 |
| `VcpNumber::{Precipitation12, ClearAirLongPulse31, ClearAirShortPulse32, ClearAir35, PrecipitationMpda112, PrecipitationSz2_212, GeneralSurveillance215}` | `VcpNumber::{Vcp12, Vcp31, Vcp32, Vcp35, Vcp112, Vcp212, Vcp215}` | Same names as `nexrad_decode::messages::VcpNumber`; see the table below. #188 |

`VcpNumber` variants are named by pattern number (#188), as in `nexrad-decode`:

| 1.0.0-rc.2 | 1.0 |
|---|---|
| `VcpNumber::Precipitation12` | `VcpNumber::Vcp12` |
| `VcpNumber::ClearAirLongPulse31` | `VcpNumber::Vcp31` |
| `VcpNumber::ClearAirShortPulse32` | `VcpNumber::Vcp32` |
| `VcpNumber::ClearAir35` | `VcpNumber::Vcp35` |
| `VcpNumber::PrecipitationMpda112` | `VcpNumber::Vcp112` |
| `VcpNumber::PrecipitationSz2_212` | `VcpNumber::Vcp212` |
| `VcpNumber::GeneralSurveillance215` | `VcpNumber::Vcp215` |
| `VcpNumber::Unknown(11)`, `Unknown(21)`, `Unknown(121)`, `Unknown(211)`, `Unknown(221)` | `VcpNumber::Vcp11`, `Vcp21`, `Vcp121`, `Vcp211`, `Vcp221` |

#### Behaviour and type changes

- **Coordinates and angles are `f64`** (#164). `Site::latitude()`/`longitude()`, `registry::SiteEntry::{latitude,
  longitude}`, `registry::nearest_site(f64, f64)`, `geo::PolarPoint::{azimuth_angle_degrees, elevation_angle_degrees}`,
  `RadarCoordinateSystem::{geo_to_polar, gate_location}` and `SweepField::value_at_polar` use `f64`. With `uom`,
  `Radial::{azimuth, azimuth_spacing, elevation_angle}`, `Sweep::elevation_angle` and
  `Site::{site_elevation, feedhorn_height}` return `uom::si::f64` quantities. Literals keep compiling; `f32` variables
  need `f64::from(x)` or `x as f64`. The angles stored on `Radial`, `Sweep`, `SweepField` and `ElevationCut` (the
  `*_degrees` accessors) remain `f32`.
- **`ElevationCut` angles are `f32`** (#182), like the other stored angles: `ElevationCut::builder` takes and
  `ElevationCut::elevation_angle_degrees()` returns `f32` (was `f64`). `Scan::sweeps_at_elevation` takes `f32`, as it
  did in 1.0.0-rc.2, so it compares directly against `Sweep::elevation_angle_degrees()`.
- **`MomentValue` gained `CfpStatus(CfpStatus)`** and is still exhaustive, so an exhaustive `match` needs a new arm. It
  is only produced for `Product::ClutterFilterPower`, and `GateStatus::from(MomentValue::CfpStatus(_))` is `NoData`.
- **`GateStatus` gained `Filtered`** (#187) and is still exhaustive, so an exhaustive `match` needs a new arm. It marks
  a gate whose valid value a processing step removed; the value is kept.
- **`GateStatus::from` takes `MomentValue` by value** (#187): `GateStatus::from(&value)` becomes
  `GateStatus::from(value)`.
- **Constructors validate** (#185). `new_empty` accepted any geometry; `SweepFieldBuilder::build` returns
  `Err(Error::InvalidGeometry { parameter, .. })` unless the azimuths are non-empty, finite, in `0.0..360.0` and
  strictly ascending, `gate_count` is non-zero, `gate_interval_km` and `azimuth_spacing_degrees` are finite and positive,
  `first_gate_range_km` is finite and not negative, and the elevation angle is finite. `CartesianField::new` and
  `VerticalField::new` reject zero dimensions, an overflowing `width * height`, and non-finite or unordered
  (`min >= max`) ranges, and the Cartesian extent must lie within valid latitudes and longitudes.
  `MomentData::from_fixed_point` requires a word size of 8 or 16, exactly `gate_count` gates of data, and a finite scale
  and offset. `RadialBuilder::build` requires a finite azimuth in `0.0..360.0`, a finite elevation and a positive
  azimuth spacing. `SiteBuilder::build` requires four ASCII alphanumeric identifier bytes and an in-range location.
- **Indexing is checked** (#185). `get` checks the row and column against their own dimensions, so `get(0, width + 3)`
  returns `None` instead of silently reading the next row; `set` and the row accessors report out-of-range indices
  instead of panicking.
- **Unscaled moments decode their sentinels** (#185). For every product except clutter filter power, raw 0 and 1 decode
  to `BelowThreshold` and `RangeFolded` even when `scale` is `0.0`; they used to decode as `Value(0.0)` and
  `Value(1.0)`.
- **`SweepField::from_radials` geometry** (#185). The elevation angle is the median over the radials carrying the
  product (consistent with `Sweep::elevation_angle_degrees`) and the azimuth spacing comes from the first of them,
  rather than both coming from the first radial in the sweep. Radials whose first gate range or gate interval differ
  from the first product-bearing radial are skipped instead of being mis-registered.
- **`SweepField::value_at_polar` respects sector coverage** (#185). The azimuth is normalized into `0.0..360.0`, the
  lookup wraps across north only when the field covers the full circle, and for a sector or partial sweep azimuths
  more than half the azimuth spacing outside the covered sector return `None` instead of the far end's value.
- **`SweepField::product()`** records which product a field holds (`Some` for fields built from radials, copied by
  `new_like`, optional on the builder).
- **Site registry membership changed** (#148). Added `KBHX`, `KDOX`, `KHDC`, `KLGX` and `PAPD`. Removed `KLIX` (replaced
  by `KHDC`), the invalid Alaska aliases `KABC`, `KACG`, `KAIH` and `KAKC` (use the `P...` identifiers), and a duplicate
  `KCCX` entry. `KFDX` and `KPBZ` coordinates were corrected. `registry::site_by_id("KLIX")` now returns `None`, and
  `nearest_site` can return a different site near those locations.
- **`geo_to_polar` returns different ranges** (#188). It is now the exact inverse of `polar_to_geo` under one model
  (spherical earth of radius 6371 km, 4/3 effective-radius refraction, antenna height above mean sea level). It used
  `ground / cos(elevation)` and ignored the antenna height, so the new range is longer, by about 3.6 km at 300 km on a
  19.5 degree tilt and by metres on the low tilts. `polar_to_geo` results move by under a metre. The new
  `beam_height_meters` and `ground_range_km` expose the same geometry.
- **`registry::nearest_site` uses great-circle distance** (#188), so near the antimeridian it can return a different
  (correct) site. Registry sites have a feedhorn height of 0; prefer `Scan::site()`, decoded from the volume data
  block, for a `RadarCoordinateSystem`.
- **Radials carry Nyquist velocity and unambiguous range** (#188). `Radial::nyquist_velocity_mps()` and
  `unambiguous_range_km()` return `Option<f32>`, filled by the `nexrad-decode` conversions. Dealias with the per-radial
  Nyquist velocity: Doppler PRF sectors vary by azimuth within a sweep.
- **Sweeps link to their elevation cut** (#188). Split cuts, SAILS, MRLE and MPDA repeat elevations, so
  `Scan::fields`, `first_field` and `sweeps_at_elevation` can return several sweeps per angle with reflectivity and
  velocity in different ones. Select by cut with `Scan::elevation_cut(&sweep)` and
  `Scan::sweeps_with_waveform(WaveformType::ContiguousSurveillance)`.
- **`Debug` output** prints the new type and variant names, `fnv1a: 0x...` instead of `sha256: "..."` for binary blobs,
  and a `product` field on `Radial`/`MomentData` moments, and `nyquist_velocity_mps`/`unambiguous_range_km` fields on `Radial`. Update any snapshot tests that capture it.

### nexrad-decode from 1.0.0-rc.3

#### Feature flags

The `nexrad-model` feature is now `model` (still on by default), and **`uom` is no longer a default feature** (#156).
If you call any `uom`-typed accessor, enable it explicitly:

```toml
# 1.0.0-rc.3
nexrad-decode = { version = "1.0.0-rc.3", default-features = false, features = ["nexrad-model"] }

# 1.0 (add "uom" to the list if you call uom-typed accessors)
nexrad-decode = { version = "1.0", default-features = false, features = ["model"] }
```

#### Error handling

- `nexrad_decode::Error` is `#[non_exhaustive]`.
- `Error::Io` and `From<std::io::Error>` are removed (#168). Nothing produced them; handle I/O errors where you do the
  I/O (`nexrad_data::Error` still converts from `std::io::Error`).
- New variant `Error::MissingCoveragePattern`, returned by `ScanBuilder::build` when no type 5 message was seen (#169).
- New variant `Error::Model(nexrad_model::Error)` (feature `model`), returned when decoded values fail a model
  constructor's validation: moment data whose gate count does not match its encoded data, a radial with an out-of-range
  azimuth, or a site with an invalid identifier or location (#185).

#### Removed and renamed items

Accessors on decoded messages follow one rule (#173; see [Naming Conventions](architecture.md#naming-conventions) and
"Accessor Naming" in the crate docs):

- The stem is the ICD field name.
- A decoded plain number carries a unit suffix: `azimuth_angle_degrees()`, `unambiguous_range_km()`,
  `calibration_constant_db()`, `site_height_meters()`, `message_size_halfwords()`.
- `<stem>_raw()` returns the wire encoding, only where it differs from the decoded value. The old `raw_<stem>()` prefix
  form is gone.
- A decoded enum or struct takes the bare stem: `radial_status()`, `build_number()`.
- With the `uom` feature, the bare stem returns a `uom` quantity: `azimuth_angle()`, `end_range()`.
- Type and variant names use UpperCamelCase acronyms: `VcpNumber`, `RdaStatus`, `Sz2Phase`.

The renames you are most likely to hit:

| 1.0.0-rc.3 | 1.0 | PR |
|---|---|---|
| `MessageHeader` public fields (`message_type`, `date`, `time`, `segment_size`, ...) | Accessors: `message_type()` / `message_type_raw()`, `date_time()`, `segment_size_raw()`, `sequence_number()`, `segment_count()`, `segment_number()`, `redundant_channel()` / `redundant_channel_raw()` | #166 |
| `MessageHeader` | `MessageHeader<'a>`; `into_owned()` gives `MessageHeader<'static>` | #166 |
| `MessageHeader::ref_from_prefix(bytes)` (zerocopy) | `MessageHeader::parse(bytes)?` | #166 |
| `size_of::<MessageHeader>()` | `MessageHeader::SIZE` | #166 |
| `MessageHeader::rda_redundant_channel()` | `redundant_channel()` | #173 |
| `message_type as u8` | `message_type.number()`; `MessageType::from_number(u8)` | #166 |
| `MessageType::{Spare1, Reserved1..5, ReservedFAARMSOnly1..5}` | `MessageType::Reserved(u8)` | #166 |
| `MessageType::RDA*` / `RPG*`, `MessageContents::RDA*` | `Rda*` / `Rpg*` (`RDAStatusData` → `RdaStatusData`, `RDAPRFData` → `RdaPrfData`) | #160 |
| `digital_radar_data::Header::azimuth_angle_raw()` | `azimuth_angle_degrees()` | #173 |
| `digital_radar_data::Header::elevation_angle_raw()` | `elevation_angle_degrees()` | #173 |
| `digital_radar_data::Header::radial_length_raw()` | `radial_length_bytes()` | #173 |
| `digital_radar_data::Header::azimuth_resolution_spacing_type()` | `azimuth_resolution_spacing()` (see below) | #173 |
| `VolumeDataBlock::latitude_raw()` / `longitude_raw()` | `latitude_degrees()` / `longitude_degrees()` | #173 |
| `VolumeDataBlock::site_height_raw()` / `tower_height_raw()` | `site_height_meters()` / `tower_height_meters()` | #173 |
| `VolumeDataBlock::volume_coverage_pattern() -> digital_radar_data::VolumeCoveragePattern` | `-> messages::VcpNumber` | #163 |
| `VolumeDataBlock::volume_coverage_pattern_number()` | `volume_coverage_pattern_raw()` | #173 |
| `digital_radar_data::VolumeCoveragePattern::{VCP12, VCP31, ..}` | `messages::VcpNumber::{Vcp12, Vcp31, Vcp32, ..}` | #160, #163 |
| `GenericDataBlock::moment_data()` / `into_moment_data()` | Call them on the `DataBlock` wrapper: `m.reflectivity_data_block().map(\|b\| b.moment_data())`; they return `Result<MomentData>` | #170 |
| `GenericDataBlock::cfp_moment_data()` / `into_cfp_moment_data()` | `moment_data()` / `into_moment_data()` on the CFP block's `DataBlock` | #170 |
| `digital_radar_data::Message::site() -> Option<Site>` | `site()? -> Option<Site>` (returns `Result<Option<Site>>`) | #185 |
| `rda_status_data::RDAStatus`, `RDABuildNumber`, `RMSControlStatus` | `RdaStatus`, `RdaBuildNumber`, `RmsControlStatus` | #160 |
| `rda_status_data::Message::raw_<stem>()` | `<stem>_raw()` (`raw_rda_build_number()` → `build_number_raw()`) | #173 |
| `rda_status_data::Message::volume_coverage_pattern() -> Option<VolumeCoveragePatternNumber>` | `-> Option<VcpSelection>`; `number()` is a `VcpNumber`, `source()` replaces `local()`/`remote()` | #163 |
| `volume_coverage_pattern::ElevationDataBlock::elevation_angle()` | `elevation_angle_degrees()` | #173 |
| `summarize` module | Removed; iterate `decode_messages(..)` and match on `message.contents()` | #168 |

The full lists are in the [nexrad-decode appendix](#nexrad-decode-100-rc3-to-10).

`summarize` has no replacement; a few lines of iteration produce the same information:

```rust
// 1.0.0-rc.3
let summary = nexrad_decode::summarize::messages(&messages);
println!("{:?}", summary.volume_coverage_patterns);

// 1.0
for message in &messages {
    match message.contents() {
        MessageContents::DigitalRadarData(m) => {
            let h = m.header();
            println!("elevation {} azimuth {}", h.elevation_number(), h.azimuth_angle_degrees());
            if let Some(volume) = m.volume_data_block() {
                println!("{}", volume.volume_coverage_pattern()); // prints "VCP 212"
            }
        }
        MessageContents::RdaStatusData(s) => println!("{:?}", s.rda_status()),
        _ => {}
    }
}
```

Scan assembly moved into this crate (#169). If you assembled a `Scan` from decoded messages yourself, use
`nexrad_decode::model::ScanBuilder` (feature `model`): `ScanBuilder::new()`, `add_message`/`add_messages`, `merge`,
`build()`.

#### Behaviour and type changes

- **Two accessor names now mean something else.** `digital_radar_data::Header::azimuth_resolution_spacing()` used to be
  the `uom` angle and now returns the `AzimuthResolutionSpacing` enum (formerly `azimuth_resolution_spacing_type()`);
  use `azimuth_resolution_spacing_degrees()` for the number. `clutter_filter_map::RangeZone::end_range()` used to
  return the `u16` kilometres and is now the `uom` length (feature `uom`); use `end_range_km()` for the number.
- **`uom` is off by default.** Code that compiled only because `uom` was a default feature now fails to find the `uom`
  accessors; enable the feature or switch to the unit-suffixed accessors.
- **VCP 32 is recognised.** `VcpNumber::Vcp32` replaces what decoded as `Unknown(32)` in the type 31 path (#163).
  `VcpNumber` also implements `Display` (`VCP 212`), which the old `VolumeCoveragePattern` enum did not.
- **VCPs 11, 21, 121, 211 and 221 are recognised** (#188): `Vcp11`, `Vcp21`, `Vcp121`, `Vcp211` and `Vcp221` replace
  `Unknown(n)`, and the variants match `nexrad_model::data::VcpNumber` one for one.
- **Converted radials carry radar physics** (#188). `digital_radar_data::Message::radial()`/`into_radial()` set the
  model radial's Nyquist velocity and unambiguous range from the radial data block (left `None` when the block is
  absent), and the legacy (type 1) conversion sets the unambiguous range from its header.
- **Negative elevation angles decode correctly** (#146). VCP elevation and sector angles with the sign bit set were
  returned as their 360-degree-wrapped value (for example 359.8 instead of -0.2).
- **Spectrum width blocks report `DataBlockName::SpectrumWidth`** instead of `Unknown(*b"SW ")` (#173).
- **With `uom`, `RadialDataBlock::unambiguous_range()` is 10× smaller**: it now applies the ICD 0.1 km scale. Use the new
  `unambiguous_range_km()` for the plain value (#173).
- **Legacy (type 1) radials with an unknown status** convert to `RadialStatus::Unknown(v)` instead of
  `IntermediateRadialData` (#173). `digital_radar_data_legacy::Message::elevation_number()` returns `u8`, and
  `radial_status()` and `vcp_number()` return decoded enums, with the codes in `radial_status_raw()` and
  `vcp_number_raw()`.
- **`MessageType` ordering.** It is no longer `#[repr(u8)]`, and its derived `Ord` puts `Reserved(_)` after
  `RdaLogData`. Compare `number()` to order by wire number.
- **Lifetimes.** `MessageHeaders::primary()`/`iter()` and `Message::header()` borrow from `self` rather than from the
  input bytes; `Message::contents()` returns `&MessageContents<'a>`.
- **No zerocopy traits** are implemented by any public `nexrad-decode` type (#166).

### nexrad-data from 1.0.0-rc.7

#### Feature flags

```toml
# 1.0.0-rc.7: real-time streaming
nexrad-data = { version = "1.0.0-rc.7", features = ["aws-polling"] }
# 1.0: streaming is part of the default `aws` feature
nexrad-data = { version = "1.0" }
```

```toml
# 1.0.0-rc.7: local files only, parallel decoding
nexrad-data = { version = "1.0.0-rc.7", default-features = false, features = ["nexrad-model", "rayon"] }
# 1.0
nexrad-data = { version = "1.0", default-features = false, features = ["model", "parallel"] }
```

- `aws-polling` is removed; real-time streaming is part of `aws` (#161).
- `nexrad-model` is renamed to `model` (#156).
- `futures`, `nexrad-model`, `once_cell`, `rayon`, `reqwest`, `tokio` and `xml` are no longer features. Use `aws`,
  `model` or `parallel`.
- `nexrad-decode`'s default features are no longer enabled through this crate, so `nexrad-decode`'s `uom` is not turned
  on for you. Add `nexrad-decode = { version = "1.0", features = ["uom"] }` if you call its `uom` accessors.

#### Error handling

`AwsError` moved and lost its `reqwest` payloads (#160, #171):

```rust
// 1.0.0-rc.7
use nexrad_data::result::aws::AWSError;
match err {
    nexrad_data::result::Error::AWS(AWSError::S3GetObjectRequest(e)) => retry(e.is_timeout()),
    nexrad_data::result::Error::AWS(AWSError::S3ObjectNotFound) => missing(),
    other => fail(other),
}

// 1.0
use nexrad_data::{AwsError, Error};
match err {
    Error::Aws(aws) if aws.is_transient() => retry(),
    Error::Aws(AwsError::S3ObjectNotFound) => missing(),
    _ => fail(),
}
```

| 1.0.0-rc.7 | 1.0 | PR |
|---|---|---|
| `nexrad_data::result::aws::AWSError` | `nexrad_data::AwsError` (defined at `nexrad_data::result::AwsError`) | #160, #171 |
| `Error::AWS(..)` | `Error::Aws(..)` | #160 |
| `AWSError::S3ListObjects(reqwest::Error)`, `S3GetObjectRequest(..)`, `S3Streaming(..)` | Same variants holding `Box<dyn std::error::Error + Send + Sync>`; `source()` returns it | #171 |
| `AWSError::PollingAsync` | Removed (never constructed) | #171 |
| `Error::Decompression(bzip2::Error)` | Removed (never constructed); bzip2 failures are `Error::Io` | #171 |
| (new) | `AwsError::is_transient()` reports whether a failure is worth retrying | #171 |

Both `Error` and `AwsError` are `#[non_exhaustive]`, and neither implements `UnwindSafe`/`RefUnwindSafe` (the boxed
source error does not); wrap them in `AssertUnwindSafe` if you need that.

#### Removed and renamed items

The `aws::realtime` module was pruned to its polling surface (#167). The timing model is crate-private, and its results
are exposed through `ChunkIterator`:

| 1.0.0-rc.7 | 1.0 | PR |
|---|---|---|
| `chunk_stream(config)` | `chunk_stream(config, \|d\| tokio::time::sleep(d))` | #161 |
| `Chunk<'a>` | `Chunk`; `Chunk::IntermediateOrEnd` holds `Record<'static>` | #167 |
| `download_chunk(site, &id) -> Result<(ChunkIdentifier, Chunk)>` | `-> Result<DownloadedChunk>`; read `d.identifier`, `d.chunk` | #167 |
| `ChunkIterator::start_with_policies(site, download, discovery)` | `ChunkIterator::start_with(PollConfig::new(site).with_download_policy(..).with_discovery_policy(..))` | #167 |
| `ChunkIterator::from_chunk(site, id, download, discovery)` | `ChunkIterator::from_chunk(config, id)` | #167 |
| `ChunkIterator::download_policy()` / `discovery_policy()` | `config().download_policy()` / `config().discovery_policy()` | #167 |
| `ChunkIterator::project_remaining_scan()` | `projected_volume_end_time()` | #167 |
| `ChunkIterator::time_until_next() -> Option<chrono::Duration>` | `-> Option<std::time::Duration>` | #167 |
| `RetryPolicy`, `PollConfig` public fields | Getters (`max_attempts()`, `initial_delay()`, `site()`, ...) and the existing `with_*` setters | #167 |
| `RetryPolicy::with_initial_delay(chrono::Duration)` / `with_max_delay(..)` | Take `std::time::Duration` | #167 |
| `ChunkIdentifier::new("KDMX".into(), ..)` | `ChunkIdentifier::new("KDMX", ..)` (`impl Into<String>`) | #167 |
| `estimate_chunk_availability_time` | `ChunkIterator::next_expected_time()` / `time_until_next()` | #167 |
| `ChunkTimingModel`, `ChunkTimingStats`, `ChunkCharacteristics`, `ElevationChunkMapper`, `ChunkMetadata`, `estimate_chunk_processing_time`, `project_scan_timing`, `project_full_scan_timing`, `ScanTimingProjection`, `ChunkProjection`, `RetryState`, `NextChunk`, `ChunkIdentifier::next_chunk` | No public replacement; `ChunkIterator` and `chunk_stream` handle sequencing, retries and timing | #167 |
| `volume::split_compressed_records(bytes)` | `volume::File::new(bytes).records()?` | #171 |
| `volume::Header::{tape_filename, extension_number, icao_of_radar}() -> Option<String>` | `-> Option<&str>` | #171 |
| `volume::File::header() -> Option<&Header>`, zerocopy traits on `volume::Header`, `size_of::<Header>()` | `header() -> Option<Header>` (by value), no zerocopy traits, `Header::SIZE` | #181 |
| `aws::archive::list_files(site, &date)` | `list_files(site, date)` | #171 |
| `aws::archive::download_file(identifier)` | `download_file(&identifier)` | #171 |

#### Behaviour and type changes

- **`Record::decompress` returns `Record<'static>`** (#171). Code that named the output lifetime (`Record<'b>`) drops
  it.
- **`DownloadedChunk`, `ChunkIteratorInit` and `LatestVolumeResult` are `#[non_exhaustive]`** (#167). You can still
  read and move their fields; build a `DownloadedChunk` with `DownloadedChunk::new(identifier, chunk, attempts)`.
  `get_latest_volume` and the new `get_latest_volume_from_hint` return `Result<LatestVolumeResult>`, whose `volume`
  field is `Option<VolumeIndex>`.
- **`VolumeIndex::new` accepts `1..=999`**; a debug assertion now rejects `0` (#167).
- **`File::scan` includes legacy (type 1) radials** (#169), matching `Record::radials()`. Legacy-only volumes now yield
  populated sweeps. `assemble_volume` accepts type 1 messages too.
- **`Site::identifier` of a scan comes from the type 31 message header** rather than the Archive II volume header ICAO
  (#169). The two agree for well-formed files; a missing header ICAO no longer produces `[0, 0, 0, 0]`.
- **`File::records()` returns `Error::TruncatedRecord`** instead of panicking on data shorter than the 24-byte volume
  header (#171).
- **Real-time volume selection** (#149, #154). `list_chunks_in_volume` reads every listing page and returns only the
  newest scan in a reused volume directory, sorted by sequence; `max_keys` now caps the returned chunks, not the S3
  request. `get_latest_volume` no longer returns an older volume when the newest one sits just after the wrap-around,
  and `ChunkIterator` waits instead of replaying an older scan left in the next volume directory.
- **Real-time volume rollover yields the Start chunk** (#183). When `ChunkIterator::try_next` or `chunk_stream`
  moves to a new volume, it now yields that volume's Start chunk (sequence 1, the volume header and VCP) and then every
  later chunk in sequence order. Previously it jumped to the newest chunk already published in the new volume, skipping
  the Start chunk and the chunks before the newest. Consumers feeding `assemble_volume` now receive every volume's Start
  chunk, so they no longer hit `Error::MissingCoveragePattern` after the first volume; consumers that skipped ahead
  should expect the extra chunks. To read a Start chunk's VCP yourself, use the new
  `volume::File::coverage_pattern_message()` on the `Chunk::Start` file.

### nexrad-process from 1.0.0-rc.1

#### Error handling

`nexrad_process::Error` is `#[non_exhaustive]`. Invalid parameters are now reported by the constructors, so
`Error::InvalidParameter` comes from `new` rather than from `process`. The new `Error::Model(nexrad_model::Error)`
reports an output the model rejects, such as a `CompositeReflectivity::compute` extent outside valid latitudes and
longitudes (#185).

#### Removed and renamed items

```rust
// 1.0.0-rc.1
let pipeline = SweepPipeline::new()
    .then(ThresholdFilter { min: Some(5.0), max: None })
    .then(MedianFilter { azimuth_kernel: 3, range_kernel: 3 })
    .then(GaussianSmooth { sigma_azimuth: 1.0, sigma_range: 1.0 });
let composite = CompositeReflectivity.compute(&scan, &fields, &coords, &extent, (512, 512))?;

// 1.0
let pipeline = SweepPipeline::new()
    .then(ThresholdFilter::new(Some(5.0), None)?)
    .then(MedianFilter::new(3, 3)?)
    .then(GaussianSmooth::new(1.0, 1.0)?);
let composite = CompositeReflectivity.compute(&fields, &coords, &extent, (512, 512))?;
```

| 1.0.0-rc.1 | 1.0 | PR |
|---|---|---|
| `ThresholdFilter { min, max }` | `ThresholdFilter::new(min, max)?`; read with `min()`, `max()` | #175 |
| `MedianFilter { azimuth_kernel, range_kernel }` | `MedianFilter::new(azimuth_kernel, range_kernel)?`; getters | #175 |
| `GaussianSmooth { sigma_azimuth, sigma_range }` | `GaussianSmooth::new(sigma_azimuth, sigma_range)?`; getters | #175 |
| `ScanDerivedProduct::compute(&self, scan, fields, coords, extent, resolution)` | `compute(&self, fields, coords, extent, resolution)` | Drop the `scan` argument in calls and in your own impls. #175 |
| `ScanProcessor` trait | Removed (no implementations); use a free function or your own trait | #175 |
| `VerticalCrossSection::new(azimuth_degrees: f32, max_range_km, max_altitude_m, ..)` | `new(azimuth_angle_degrees: f64, max_range_km, max_altitude_meters, ..)`; only the type of the first argument affects callers | #164, #182 |
| `VerticalCrossSection::compute(&fields)` | `compute(&fields, &coordinate_system)`, for example with `scan.coordinate_system()` | The vertical axis is altitude above mean sea level, including the antenna height, and the horizontal axis is ground range. #188 |
| `StormCell::elevation_degrees()` | `StormCell::elevation_angle_degrees()` | #182 |
| `StormCell::max_reflectivity_azimuth_degrees()` | `StormCell::max_reflectivity_azimuth_angle_degrees()` | #182 |

#### Behaviour and type changes

- `MedianFilter::new` rejects zero or even kernel sizes, `GaussianSmooth::new` rejects sigmas that are not finite and
  positive (NaN and infinity used to be accepted), and `ThresholdFilter::new` rejects a NaN bound or `min > max`.
- **Masked gates are `Filtered`** (#187). `ThresholdFilter` and `CorrelationCoefficientFilter` mark the gates they
  remove `GateStatus::Filtered` and keep their values; they used to set `NoData` with `0.0`. Code that looks for
  `NoData` to find masked gates should look for `Filtered`; code that checks for `Valid` is unaffected. Smoothing and
  storm-relative velocity pass `Filtered` gates through, and composite reflectivity, cross-sections and cell detection
  skip them, as they do `NoData`.
- The processors now implement `Debug`, `Clone` and `PartialEq` (and `CompositeReflectivity` also `Copy` and
  `Default`).

### nexrad-render from 1.0.0-rc.4

#### Error handling

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `Error::NoRadials` | `Error::EmptyField`, also returned for Cartesian or vertical fields with no rows or columns | #174 |
| `Error::ImageSave(String)` | `Error::Image(image::ImageError)`; `source()` returns it | #174 |
| `Error::ProductNotFound` | Removed (only `render_radials` produced it) | #174 |
| `Error::InvalidDimensions` | Unchanged; also returned for a zero width or height | #174 |

`nexrad_render::Error` is `#[non_exhaustive]` and, because `image::ImageError` is not, no longer implements
`UnwindSafe`/`RefUnwindSafe`.

#### Removed and renamed items

Render through a `SweepField`:

```rust
// 1.0.0-rc.4
let image = render_radials(sweep.radials(), Product::Reflectivity, &nws_reflectivity_scale(), &options)?;

// 1.0
let field = sweep
    .field(Product::Reflectivity)
    .ok_or_else(|| std::io::Error::other("no reflectivity"))?;
let image = render_sweep(&field, &default_scale(Product::Reflectivity), &options)?.into_image();
```

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `render_radials(radials, product, &DiscreteColorScale, &options) -> RgbaImage` | `render_sweep(&SweepField::from_radials(radials, product)?, &scale.into(), &options)?.into_image()` | #174 |
| `render_radials_default(radials, product, &options)` | `render_sweep(&field, &default_scale(product), &options)` | #174 |
| `default_color_scale(product)` | `default_scale(product)` | #174 |
| `default_scale(product) -> DiscreteColorScale` | `default_scale(product) -> ColorScale` | #174 |
| `ColorLookupTable::from_scale(&DiscreteColorScale, ..)`, `from_color_scale(&ColorScale, ..)` | `ColorLookupTable::new(&ColorScale, min, max, size)` | Convert a `DiscreteColorScale` with `.into()`. #174 |
| `RenderOptions::new(usize, usize)`, `size() -> (usize, usize)` | `u32` | #174 |
| `RenderResult::new(..)` | Crate-private | #174 |
| `RenderMetadata::{center_pixel, pixels_per_km, max_range_km, polar_to_pixel, pixel_distance_to_km, km_to_pixel_distance}` returning plain values | Return `Option`, `Some` only for polar renders | #174 |
| `RenderMetadata::elevation_degrees() -> Option<f32>` | `RenderMetadata::elevation_angle_degrees() -> Option<f64>` | #164, #182 |
| `PointQuery { polar, geo, value, status }` struct literal | Read `PointQuery` from `RenderResult::query_*`; it is `#[non_exhaustive]`, and `polar` is a `PolarPoint` with `azimuth_angle_degrees` and `elevation_angle_degrees` fields | #182 |

#### Behaviour and type changes

- **Metadata is projection-aware** (#174). `RenderMetadata::projection()` returns `Projection::Polar`,
  `Projection::Geographic` or `Projection::Vertical`. Polar-only accessors and `RenderResult::query_pixel`,
  `query_polar` and `query_geo` return `None` for `render_cartesian` and `render_vertical` results, which used to report
  `pixels_per_km = 0` and divide by zero in `pixel_to_polar`. For `render_cartesian` results, `pixel_to_geo` and
  `geo_to_pixel` now map linearly through the extent. The `Polar`, `Geographic` and `Vertical` variants are
  `#[non_exhaustive]`, so a struct pattern on them needs `..`: `Projection::Polar { center_pixel, .. }`. The `Polar`
  elevation field is `elevation_angle_degrees` and the `Vertical` altitude field is `altitude_range_meters` (#182).
- **Polar angles are `f64`** (#164): `RenderMetadata::elevation_angle_degrees()`, `polar_to_pixel` and
  `RenderResult::query_polar` take or return `f64`.
- **Image dimensions are `u32`** (#174). `RenderOptions::native_for` saturates at `u32::MAX`.
- **`ColorLookupTable::new` clamps sizes below 2 to 2**; the old constructors divided by zero or produced NaN.
- **Colour range for `render_sweep`** is unchanged: the lookup table spans the field's data range
  (`SweepField::value_range`). For a field with no valid gates it now falls back to the product's range before the
  reflectivity default; the image is the same because no gate is coloured.
- **`RenderOptions::with_extent`** is unchanged: for `render_sweep` it sets the extent reported by
  `RenderMetadata::geo_extent()`, while the image itself stays centred on the radar and scaled to the field's maximum
  range.
- `nexrad_render::image` re-exports the `image` crate, so you can name `image::ImageError` without your own dependency.
- **Queries on a sector sweep** (`RenderResult::query_pixel`, `query_polar`, `query_geo`) return `None` beyond the
  covered azimuths instead of the far end's value across north (#185). `Error::EmptyField` is no longer reachable for
  Cartesian or vertical fields built with `new`, which rejects zero dimensions.

### nexrad from 1.0.0-rc.4

#### Feature flags

```toml
# 1.0.0-rc.4
nexrad = { version = "1.0.0-rc.4", default-features = false, features = ["aws-polling"] }

# 1.0
nexrad = { version = "1.0", default-features = false, features = ["aws"] }
```

- `aws-polling` and `tokio` are removed; `stream` is part of `aws` (#156, #161).
- Sub-crates are depended on with default features off; each facade feature enables what it needs. `decode` implies
  `model`, `data` implies `decode` and `model`, `parallel` implies `data`, `aws` implies `data` and `chrono`. A minimal
  local-file build is `features = ["data"]` and no longer compiles reqwest.
- `full` (the default) is `wasm` + `parallel`.

#### Error handling

Errors raised inside `nexrad-data` are flattened by layer (#176):

```rust
// 1.0.0-rc.4
match nexrad::load_file(path) {
    Ok(scan) => use_scan(scan),
    Err(nexrad::Error::Io(e)) => unreadable(e), // reading the file
    Err(nexrad::Error::Data(nexrad::data::result::Error::Decode(e))) => malformed(e),
    Err(nexrad::Error::Data(e)) => archive_error(e),
    Err(e) => other(e),
}

// 1.0
match nexrad::load_file(path) {
    Ok(scan) => use_scan(scan),
    Err(nexrad::Error::Io(e)) => unreadable(e),
    Err(nexrad::Error::Decode(e)) => malformed(e), // no longer nested in Error::Data
    Err(nexrad::Error::Data(e)) => archive_error(e),
    Err(e) => other(e), // nexrad::Error is #[non_exhaustive]
}
```

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `Error::Data(data::Error::Decode(e))` | `Error::Decode(e)` | #176 |
| `Error::Data(data::Error::Model(e))` | `Error::Model(e)` | #176 |
| `Error::Data(data::Error::Io(e))` | `Error::Io(e)` | #176 |
| `Error::NoDataAvailable { date: String, .. }` | `date: chrono::NaiveDate`, and the variant exists only with `aws` | #176 |
| `Error::MissingSiteMetadata` | Removed (only `coordinate_system_required` produced it) | #172 |

`Display` text changes with the flattening: `data error: error decoding NEXRAD data` becomes
`decode error: <decode message>`, and so on. Items from `stream` are flattened the same way.

#### Removed and renamed items

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `extract_field(sweep, product)` | `sweep.field(product)` | #172 |
| `extract_fields(&scan, product)` | `scan.fields(product)` | #172 |
| `extract_first_field(&scan, product)` | `scan.first_field(product)` | #172 |
| `coordinate_system(&scan)` | `scan.coordinate_system()` | #172 |
| `coordinate_system_required(&scan)?` | `scan.coordinate_system().ok_or(/* your error */)?` | #172 |
| `sites()` | `nexrad::model::meta::registry::sites()` | #172 |
| `site(id)` | `nexrad::model::meta::registry::site_by_id(id)` | #172 |
| `nearest_site(lat: f32, lon: f32)` | `nexrad::model::meta::registry::nearest_site(lat: f64, lon: f64)` | #164, #172 |
| `download(identifier)` | `download(&identifier)` | #176 |
| `stream(site)` | `stream(site, \|d\| tokio::time::sleep(d))` | #161 |
| (your own `chrono` dependency) | `nexrad::chrono` (feature `chrono`, implied by `aws`) | #176 |

### Serialized data

**Model data serialized with serde by a release candidate does not deserialize with 1.0. Regenerate it from the Archive
II source files.** There is no compatibility shim. The incompatible changes are:

| Type | Change | PR |
|---|---|---|
| `WaveformType` | Variants serialize as `"ContiguousSurveillance"`, `"ContiguousDopplerWithAmbiguityResolution"`, `"ContiguousDopplerWithoutAmbiguityResolution"`, `"Batch"`, `"StaggeredPulsePair"` instead of `"CS"`, `"CDW"`, `"CDWO"`, `"B"`, `"SPP"` | #160 |
| `ChannelConfiguration` | `"Sz2Phase"` instead of `"SZ2Phase"` | #160 |
| `RadialStatus` | `"ElevationStartVcpFinal"` instead of `"ElevationStartVCPFinal"` | #160 |
| `MomentData` | A struct with a `product` field next to the encoding fields; it was `serde(transparent)` over an inner block. Every moment on a `Radial`, including `clutter_filter_power`, has this shape | #170 |
| `SweepField` | Gains a `product` field | #170 |
| `Site` | `latitude` and `longitude` are `f64` (were `f32`). Self-describing formats such as JSON still read old values; fixed-width binary formats such as bincode do not | #164 |
| `Site` | `height_meters` and `tower_height_meters` are renamed `site_elevation_meters` and `feedhorn_height_meters` | #182 |
| `Radial` | `collection_timestamp` is renamed `collection_timestamp_millis` | #182 |
| `Radial` | Gains `nyquist_velocity_mps` and `unambiguous_range_km` (`Option<f32>`). Self-describing formats such as JSON read data without them as `None`; fixed-width binary formats such as bincode do not | #188 |
| `SweepField` | `elevation_degrees` and `azimuths` are renamed `elevation_angle_degrees` and `azimuth_angles_degrees` | #182 |
| `VolumeCoveragePattern` | `doppler_velocity_resolution`, `sails_cuts` and `mrle_cuts` are renamed `doppler_velocity_resolution_mps`, `sails_cut_count` and `mrle_cut_count` | #182 |
| `ElevationCut` | `elevation_angle_degrees` is `f32` (was `f64`) | #182 |
| `PolarPoint` | `azimuth_degrees` and `elevation_degrees` are renamed `azimuth_angle_degrees` and `elevation_angle_degrees` | #182 |
| `MomentData` | `first_gate_range`, `gate_interval` and `values` are renamed `first_gate_range_meters`, `gate_interval_meters` and `raw_values`, matching the accessors | #PR |
| `MomentData` | `raw_values` serializes as a byte string (`serialize_bytes`) instead of a sequence of integers. JSON and other formats without a bytes type still write and read an array of integers; formats with one, such as MessagePack and CBOR, write a binary string | #PR |
| `SweepField`, `CartesianField`, `VerticalField` | `status` is renamed `statuses`, matching the accessor | #PR |
| `CfpStatus` | Variants serialize as their PascalCase names (`"FilterNotApplied"`, `"PointClutterFilterApplied"`, `"DualPolOnlyFilterApplied"`, `{"Reserved": n}`) instead of snake_case (`"filter_not_applied"`, ...) | #PR |
| `SweepField`, `CartesianField`, `VerticalField`, `MomentData`, `Radial`, `Site`, `Sweep` | Deserialization applies the constructors' validation and fails on data they reject: field `values` and `statuses` must each hold one entry per cell, `raw_values` must hold exactly `gate_count` gates of `data_word_size` bits, each `Radial` moment must sit under its own product's key, `Site` coordinates must be in range, and every radial of a `Sweep` must share its `elevation_number` | #PR |

Unchanged: `VcpNumber` serializes as its `u16` number (as `VCPNumber` already did), and the field names of `Scan` and
`Sweep` are the same. From 1.0, every serialized field name equals its accessor's name, every enum variant serializes
as its PascalCase Rust name, and this representation is stable within 1.x. The `nexrad-model/tests/snapshots/` files
show the 1.0 serialized form of a real scan.

Data serialized by `nexrad-model` 0.1.0 is further away: its `Scan` held a `coverage_pattern_number` instead of a
`coverage_pattern`, its `Site` had `height_meters` where 1.0 has `site_elevation_meters`, its `RadialStatus` used
`VolumeScanStart`/`VolumeScanEnd`, and its `MomentData` held only scale, offset and values. Regenerate it as well.

`Debug` output also changed (see the [nexrad-model](#nexrad-model-from-100-rc2) section); it was never a stable
format.

## Upgrading from 0.x

Read this section for the crate split and the mapping of 0.x entry points, then the
[release-candidate section](#upgrading-from-a-release-candidate) for each crate you use: every release-candidate change
also applies to you, and those tables are not repeated here.

### The crate split

`nexrad` 0.0.3 was a single crate. In 1.0 its responsibilities live in five crates, and `nexrad` is a facade over them:

| `nexrad` 0.0.3 module | 1.0 home | What it does now |
|---|---|---|
| `nexrad::download` (feature `download`, `aws-sdk-s3`) | `nexrad_data::aws::archive`, or `nexrad::list_scans` / `download` / `download_latest` / `download_at` (feature `aws`) | Plain HTTPS through reqwest, no AWS SDK or credentials; runtime-agnostic, WASM-compatible |
| `nexrad::decompress` | `nexrad_data::volume` (`Record::decompress`), applied automatically by `File::scan` and `nexrad::load` | bzip2 LDM record decompression |
| `nexrad::decode` | `nexrad_decode::messages` (protocol level) or `nexrad::load` / `nexrad_data::volume::File::scan` (model level) | ICD message parsing for every message type the library supports |
| `nexrad::model` | `nexrad_decode::messages` (wire-format structs) and `nexrad_model::data` (`Scan`, `Sweep`, `Radial`, `MomentData`) | See [Conceptual changes](#conceptual-changes-since-0x) |
| `nexrad::file` | `nexrad_data::aws::archive::Identifier`, `nexrad_data::volume::{File, Record}` | File identifiers and volume handling |
| `nexrad::result` | `nexrad::Error` and each sub-crate's `Error` | One error type per crate, unified in the facade |
| (the `render` example) | `nexrad_render` | Rendering is a library crate now |
| (none) | `nexrad_process` | Filters and derived products |

`nexrad-model` 0.1.0, `nexrad-decode` 0.1.2 and `nexrad-data` 0.2.0 were the first versions after the split and already
have the 1.0 shape. Their sections below list what changed beyond the release-candidate tables.

Feature flags:

| 0.x | 1.0 |
|---|---|
| `nexrad` 0.0.3 `download` | `nexrad` `aws` (or `nexrad-data` `aws`) |
| `nexrad-data` 0.2.0 `decode` | Removed: decoding is always available |
| `nexrad-data` 0.2.0 `nexrad-model` | `model` |
| `nexrad-data` 0.2.0 `aws` (pulled in tokio) | `aws` (no tokio; real-time streaming included) |
| `nexrad-decode` 0.1.2 `nexrad-model` | `model` (default) |
| `nexrad-decode` 0.1.2 `uom` (default) | `uom` (opt-in) |
| `nexrad-model` 0.1.0 `serde`, `chrono`, `uom` | Unchanged |

### nexrad 0.0.3

| 0.0.3 item | 1.0 item |
|---|---|
| `decode::decode_file(&Vec<u8>) -> Result<model::DataFile>` | `nexrad::load(&[u8]) -> Result<Scan>` (decompresses as needed), or `nexrad_decode::messages::decode_messages(&[u8])` for decompressed message bytes |
| `decompress::decompress_file(&[u8]) -> Result<Vec<u8>>` | Not needed with `load`/`File::scan`; per record, `nexrad_data::volume::File::new(bytes).records()?` then `Record::decompress()` |
| `file::is_compressed(&[u8])` | `nexrad_data::volume::Record::compressed()` per LDM record. (`File::compressed()` reports whole-file gzip compression, which 0.0.3 did not handle.) |
| `download::list_files(&str, &NaiveDate) -> Result<Vec<FileMetadata>>` | `nexrad::list_scans(&str, NaiveDate) -> Result<Vec<Identifier>>` or `nexrad_data::aws::archive::list_files(&str, NaiveDate)` |
| `download::download_file(&FileMetadata) -> Result<Vec<u8>>` | `nexrad::download(&Identifier) -> Result<Scan>`, or `nexrad_data::aws::archive::download_file(&Identifier) -> Result<File>` |
| `file::FileMetadata` (`site()`, `date()`, `identifier()`) | `nexrad_data::aws::archive::Identifier` (`site()`, `date_time()`, `name()`) |
| `model::DataFile` (`volume_header()`, `elevation_scans()`) | `nexrad_model::data::Scan` (`site()`, `coverage_pattern()`, `sweeps()`); the volume header is `nexrad_data::volume::File::header()` |
| `model::Message31` (`header()`, `reflectivity_data()`, `velocity_data()`, `sw_data()`, `zdr_data()`, `phi_data()`, `rho_data()`, `cfp_data()`, `volume_data()`, `elevation_data()`, `radial_data()`) | Model level: `nexrad_model::data::Radial` (`reflectivity()`, `velocity()`, `spectrum_width()`, `differential_reflectivity()`, `differential_phase()`, `correlation_coefficient()`, `clutter_filter_power()`, `moment(product)`). Protocol level: `nexrad_decode::messages::digital_radar_data::Message` (`header()`, `*_data_block()`) |
| `model::Message31Header` (`azm()`, `elev()`, `elev_num()`, `radial_status()`, `ray_date()`, `ray_time()`, ...) | `Radial::azimuth_angle_degrees()`, `elevation_angle_degrees()`, `elevation_number()`, `radial_status()`, `collection_time()`; or `digital_radar_data::Header` accessors |
| `model::DataMoment` / `GenericData` (`moment_data()`, `scale()`, `offset()`, `number_data_moment_gates()`, ...) | `nexrad_model::data::MomentData` (`iter()`/`values()` decode gates to `MomentValue`; `scale()`, `offset()`, `gate_count()`, `raw_values()`) |
| `model::VolumeData`, `ElevationData`, `RadialData` | `nexrad_decode::messages::digital_radar_data::{VolumeDataBlock, ElevationDataBlock, RadialDataBlock}`; site position is also `Scan::site()` |
| `model::MessageHeader`, `VolumeHeaderRecord`, `DataBlockHeader` | `nexrad_decode::messages::MessageHeader`, `nexrad_data::volume::Header`, `digital_radar_data::DataBlockId` |
| `result::Error::{FileError, DeserializationError, DecompressionError}` | `nexrad::Error::Io`, `Decode`, `Data` |
| `result::Error::{S3GeneralError, S3ListObjectsError, S3GetObjectError, S3StreamingError}` | `nexrad::Error::Data(nexrad_data::Error::Aws(AwsError::..))` and `nexrad::Error::NoDataAvailable` |

### nexrad-model 0.1.0

| 0.1.0 item | 1.0 item |
|---|---|
| `Radial::new(..)` | `Radial::builder(..)` (see the [release-candidate table](#nexrad-model-from-100-rc2)) |
| `Radial::specific_differential_phase()` | Removed; `Radial::clutter_filter_power()` and `Radial::moment(Product)` are new |
| `MomentData::from_fixed_point(scale, offset, values)` | `MomentData::from_fixed_point(product, gate_count, first_gate_range, gate_interval, data_word_size, scale, offset, values)` |
| `MomentData::values()` | Unchanged; `iter()` avoids the allocation. `MomentValue` gains `CfpStatus` |
| `RadialStatus::{VolumeScanStart, VolumeScanEnd, ElevationStartVCPFinal}` | `ScanStart`, `ScanEnd`, `ElevationStartVcpFinal`; new `Unknown(u8)` |
| `Scan::new(coverage_pattern_number: u16, sweeps)` | `Scan::new(VolumeCoveragePattern, sweeps)` or `Scan::with_site(site, vcp, sweeps)` |
| `Scan::coverage_pattern_number() -> u16` | `-> VcpNumber`; `.number()` gives the `u16` |
| `Scan::sweeps() -> &Vec<Sweep>`, `Sweep::radials() -> &Vec<Radial>` | Return slices |
| `Site::new(id, lat: f32, lon: f32, height, feedhorn_height)` | `Site::builder(id, lat: f64, lon: f64).site_elevation_meters(..).feedhorn_height_meters(..).build()` |
| `Site::latitude()`/`longitude() -> f32` | `-> f64` |
| `Site::height_meters()` / `height()` | `site_elevation_meters()` / `site_elevation()` |
| `Site::feedhorn_height_meters()` / `feedhorn_height()` | Unchanged |
| `uom` accessors (`Radial::azimuth()`, `Site::feedhorn_height()`, ...) returning `uom::si::f32` | `uom::si::f64` |
| `Error::ElevationMismatchError` | `Error::ElevationMismatch` |

New in 1.0: `Product`, `VolumeCoveragePattern`, `ElevationCut`, `VcpNumber`, `SweepField`, `CartesianField`,
`VerticalField`, `GateStatus`, the `geo` module and the `meta::registry` site registry.

### nexrad-decode 0.1.2

0.1.2 decoded from a reader; 1.0 decodes from a byte slice and borrows from it:

```rust
// 0.1.2
let messages = nexrad_decode::messages::decode_messages(&mut std::io::Cursor::new(&bytes))?;

// 1.0
let messages = nexrad_decode::messages::decode_messages(&bytes)?;
```

| 0.1.2 item | 1.0 item |
|---|---|
| `messages::decode_messages<R: Read + Seek>(&mut R) -> Result<Vec<Message>>` | `messages::decode_messages(&[u8]) -> Result<Vec<Message<'_>>>` |
| `messages::decode_message_header(&mut R)` | `messages::MessageHeader::parse(&[u8])` |
| `messages::decode_message_contents(&mut R, MessageType)`, `digital_radar_data::decode_digital_radar_data`, `rda_status_data::decode_rda_status_message`, `volume_coverage_pattern::decode_volume_coverage_pattern`, `clutter_filter_map::decode_clutter_filter_map` | `decode_messages`, which decodes every supported message type |
| `messages::message_header::MessageHeader` (module) | `messages::MessageHeader` |
| `MessageContents::Other` for most types | Typed variants for most message types (`DigitalRadarDataLegacy`, `ConsoleMessage`, `ClutterCensorZones`, `RequestForData`, ...); `Other` remains for the rest |
| Public struct fields on `digital_radar_data::{Message, Header, VolumeDataBlock, ..}` | Accessor methods: `m.header()`, `m.volume_data_block()`, `m.reflectivity_data_block()`, ... |
| `digital_radar_data::Message::specific_diff_phase_data_block` | Removed |
| `digital_radar_data::ScaledMomentValue`, `GenericDataBlock::decoded_values()` | `DataBlock::moment_data()?` then `MomentData::iter()` (feature `model`) |
| `summarize` (`messages`, `MessageSummary`, `ScanSummary`) | Removed; see [the release-candidate section](#nexrad-decode-from-100-rc3) |
| `Error::FileError`, `DeserializationError` | Removed; decoding no longer does I/O or uses bincode |
| `Error::DecodingError(String)`, `MessageMissingDateError` | `Error::Decoding(String)`, `MessageMissingDate` |

The 0.1.2 accessor names (`azimuth_angle()`, `latitude()`, `lrtup()`, `site_height()`, ...) follow the 1.0 convention
after the renames listed in the [release-candidate section](#nexrad-decode-from-100-rc3) and
[appendix](#nexrad-decode-100-rc3-to-10): plain numbers gain a unit suffix and `uom` quantities keep the bare name.

### nexrad-data 0.2.0

| 0.2.0 item | 1.0 item |
|---|---|
| `volume::File::header() -> Result<Header>` | `-> Option<&Header>` |
| `volume::File::records() -> Vec<Record>` | `-> Result<Vec<Record>>` |
| `volume::File::data() -> &Vec<u8>` | `-> &[u8]` |
| `volume::Record::messages() -> Result<Vec<Message>>` | `-> Result<Vec<Message<'_>>>`; `Record::radials()` builds model radials |
| `volume::split_compressed_records(&[u8])` | `File::new(bytes).records()?` |
| `aws::archive::list_files(site, &date)` / `download_file(identifier)` | `list_files(site, date)` / `download_file(&identifier)` |
| `aws::realtime::poll_chunks(site, Sender, Option<Sender<PollStats>>, Receiver<bool>)` | `aws::realtime::chunk_stream(PollConfig::new(site), sleep)` (a `Stream`) or `ChunkIterator` (pull-based) |
| `aws::realtime::PollStats`, `NewChunkStats` | `ChunkIterator::requests_made()`, `bytes_downloaded()`; `DownloadedChunk::attempts` |
| `aws::realtime::estimate_next_chunk_time(&id)` | `ChunkIterator::next_expected_time()` / `time_until_next()` |
| `ChunkIdentifier::new(site, volume, name, date_time)` | `ChunkIdentifier::from_name(site, volume, name, upload_date_time)?`; `new` takes the parsed parts |
| `ChunkIdentifier::date_time()` | `upload_date_time()`; `date_time_prefix()` for the time encoded in the name |
| `ChunkIdentifier::sequence() -> Option<usize>`, `chunk_type() -> Option<ChunkType>` | Return `usize` and `ChunkType` |
| `ChunkIdentifier::with_sequence(n)`, `next_chunk()`, `name_prefix()` | `with_sequence(n, chunk_type)`; sequencing is handled by `ChunkIterator` and `chunk_stream` |
| `download_chunk(site, &id) -> Result<(ChunkIdentifier, Chunk)>` | `-> Result<DownloadedChunk>` |
| `result::Error::{FileError, CompressedDataError, UncompressedDataError, AWS}` | `Error::{Io, CompressedData, UncompressedData, Aws}` |
| `result::Error::{DeserializationError, DecompressionError}` | Removed |
| `result::aws::AWSError::*Error` variants (`S3ListObjectsError`, `DateTimeError`, ...) | `AwsError::*` without the suffix (`S3ListObjects`, `DateTime`, ...); `PollingAsyncError` removed |

### Conceptual changes since 0.x

- **Model, not messages.** 0.0.3 gave you `DataFile::elevation_scans()`, a `HashMap<u8, Vec<Message31>>` of raw type 31
  messages keyed by elevation number, and you applied scale and offset to raw gate bytes yourself. In 1.0,
  `nexrad::load` returns a `Scan` with ordered `sweeps()`, each holding `Radial`s whose moments are `MomentData`
  values; `MomentData::iter()` yields decoded `MomentValue`s (`Value(f32)`, `BelowThreshold`, `RangeFolded`,
  `CfpStatus`). `Sweep::field(product)` turns a sweep into a `SweepField`, a dense azimuth × gate grid that processing
  and rendering consume. Protocol-level access is still there in `nexrad_decode::messages` when you need it.
- **Many more message types are decoded**, including legacy type 1 radials, which `File::scan` now uses alongside
  type 31.
- **Downloads use plain HTTPS.** The `aws-sdk-s3` client (with its credentials, region configuration and tokio
  requirement) is gone. `nexrad_data::aws` talks to the public NOAA buckets through reqwest, works on any async runtime,
  and builds for `wasm32`.
- **Real-time data is a stream.** The channel-based `poll_chunks` became `chunk_stream(config, sleep)`, a `Stream` of
  `DownloadedChunk`s with retry policies, and `assemble_volume` turns a volume's chunks into a `Scan`. The facade's
  `nexrad::stream(site, sleep)` wraps it.

## Worked examples

Each flow is shown for the 0.x releases where they had an equivalent, the last release candidate and 1.0. The 1.0
versions are compiled against the 1.0 crates; the older versions are taken from the documentation and examples of those
releases.

### Load a local Archive II file

**0.x: `nexrad` 0.0.3**

```rust
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::file::is_compressed;

fn main() -> nexrad::result::Result<()> {
    let mut data = std::fs::read("KDMX20220305_232324_V06")?;
    if is_compressed(&data) {
        data = decompress_file(&data)?;
    }

    let decoded = decode_file(&data)?;
    for (elevation_number, radials) in decoded.elevation_scans() {
        for radial in radials {
            if let Some(reflectivity) = radial.reflectivity_data() {
                let header = reflectivity.data();
                // Raw gate bytes; apply (raw - offset) / scale yourself, 0 = below threshold, 1 = range folded.
                let raw: &[u8] = reflectivity.moment_data();
                println!("elevation {} azimuth {} {} gates (scale {}, offset {})",
                    elevation_number, radial.header().azm(), raw.len(), header.scale(), header.offset());
            }
        }
    }
    Ok(())
}
```

**0.x: `nexrad-data` 0.2.0 with `nexrad-model` 0.1.0**

```rust
use nexrad_data::volume::File;
use nexrad_model::data::MomentValue;

fn main() -> nexrad_data::result::Result<()> {
    let file = File::new(std::fs::read("KDMX20220305_232324_V06")?);
    let scan = file.scan()?;
    println!("VCP {} with {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());

    for sweep in scan.sweeps() {
        for radial in sweep.radials() {
            if let Some(reflectivity) = radial.reflectivity() {
                for value in reflectivity.values() {
                    match value {
                        MomentValue::Value(dbz) => { /* ... */ }
                        MomentValue::BelowThreshold | MomentValue::RangeFolded => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

**1.0.0-rc.4 (`nexrad` facade)**

```rust
use nexrad::model::data::{DataMoment, MomentValue};

fn main() -> nexrad::Result<()> {
    let scan = nexrad::load_file("KDMX20220305_232324_V06")?;
    println!("{} with {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());

    for sweep in scan.sweeps() {
        for radial in sweep.radials() {
            if let Some(reflectivity) = radial.reflectivity() {
                println!("{} gates", reflectivity.gate_count()); // DataMoment trait method
                for value in reflectivity.iter() {
                    match value {
                        MomentValue::Value(dbz) => { /* ... */ }
                        MomentValue::BelowThreshold | MomentValue::RangeFolded => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

**1.0**

```rust
use nexrad::model::data::{MomentValue, Product};

fn main() -> nexrad::Result<()> {
    let scan = nexrad::load_file("KDMX20220305_232324_V06")?;
    println!("{} with {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());

    for sweep in scan.sweeps() {
        for radial in sweep.radials() {
            // `radial.reflectivity()` is shorthand for `radial.moment(Product::Reflectivity)`.
            let Some(reflectivity) = radial.moment(Product::Reflectivity) else {
                continue;
            };
            println!("{} gates", reflectivity.gate_count()); // inherent, no trait import
            for value in reflectivity.iter() {
                match value {
                    MomentValue::Value(dbz) => { let _ = dbz; }
                    MomentValue::BelowThreshold | MomentValue::RangeFolded => {}
                    MomentValue::CfpStatus(_) => {} // only for Product::ClutterFilterPower
                }
            }
        }
    }

    // Or take a whole sweep as an azimuth x gate grid:
    if let Some(field) = scan.sweeps()[0].field(Product::Reflectivity) {
        if let Some((value, status)) = field.get(0, 0) {
            println!("{value} {status:?}");
        }
    }
    Ok(())
}
```

Without the facade, `nexrad_data::volume::File::new(std::fs::read(path)?).decompress()?.scan()?` returns the same
`Scan` (`decompress` handles gzip-wrapped files; LDM records are decompressed by `scan`).

### List and download from AWS

**0.x: `nexrad` 0.0.3** (feature `download`, with tokio)

```rust
use chrono::NaiveDate;
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;

#[tokio::main]
async fn main() -> nexrad::result::Result<()> {
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).expect("is valid date");
    let metas = list_files("KDMX", &date).await?;
    if let Some(meta) = metas.last() {
        let mut data = download_file(meta).await?;
        if is_compressed(&data) {
            data = decompress_file(&data)?;
        }
        let decoded = decode_file(&data)?;
        println!("{} elevations", decoded.elevation_scans().len());
    }
    Ok(())
}
```

**0.x: `nexrad-data` 0.2.0** (archive, then real-time)

```rust
use chrono::NaiveDate;
use nexrad_data::aws::archive::{download_file, list_files};
use nexrad_data::aws::realtime::{poll_chunks, Chunk, ChunkIdentifier};
use std::sync::mpsc;

#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).expect("is valid date");
    let ids = list_files("KDMX", &date).await?;
    if let Some(id) = ids.last() {
        let file = download_file(id.clone()).await?;
        let scan = file.scan()?;
        println!("{} sweeps", scan.sweeps().len());
    }

    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (_stop_tx, stop_rx) = mpsc::channel::<bool>();
    tokio::task::spawn(async move {
        poll_chunks("KDMX", update_tx, None, stop_rx).await.expect("polls chunks");
    });
    while let Ok((chunk_id, chunk)) = update_rx.recv() {
        println!("{} ({} bytes)", chunk_id.name(), chunk.data().len());
    }
    Ok(())
}
```

**1.0.0-rc.4 (`nexrad` facade, feature `aws-polling`) and 1.0.0-rc.7 (`nexrad-data`)**

```rust
use chrono::NaiveDate; // your own chrono dependency
use futures::StreamExt;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).expect("is valid date");
    let scans = nexrad::list_scans("KDMX", date).await?;
    if let Some(id) = scans.last() {
        let scan = nexrad::download(id.clone()).await?;
        println!("{} sweeps", scan.sweeps().len());
    }

    let mut stream = std::pin::pin!(nexrad::stream("KDMX"));
    while let Some(chunk) = stream.next().await {
        println!("{}", chunk?.identifier.name());
    }
    Ok(())
}

// nexrad-data 1.0.0-rc.7 directly:
// let ids = nexrad_data::aws::archive::list_files("KDMX", &date).await?;
// let file = nexrad_data::aws::archive::download_file(ids[0].clone()).await?;
// let stream = nexrad_data::aws::realtime::chunk_stream(PollConfig::new("KDMX"));
```

**1.0** (`nexrad` with `aws`; add `tokio` and `futures-util` to your own dependencies)

```rust
use futures_util::StreamExt;
use nexrad::chrono::NaiveDate;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).expect("valid date");

    let scans = nexrad::list_scans("KDMX", date).await?;
    if let Some(id) = scans.last() {
        println!("downloading {}", id.name());
        let scan = nexrad::download(id).await?; // by reference, no clone
        println!("{} sweeps", scan.sweeps().len());
    }
    // Or in one call: nexrad::download_latest("KDMX", date).await?

    // Real-time: pass the runtime's sleep function.
    let stream = nexrad::stream("KDMX", |d| tokio::time::sleep(d));
    let mut stream = std::pin::pin!(stream);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("{} ({} attempts)", chunk.identifier.name(), chunk.attempts);
    }
    Ok(())
}
```

With `nexrad-data` directly, the archive calls are `list_files("KDMX", date)` and `download_file(id).await?.decompress()?`
followed by `.scan()?`. For real time, configure retries on a `PollConfig` and assemble chunks into scans:

```rust
use futures_util::StreamExt;
use nexrad::data::aws::realtime::{assemble_volume, chunk_stream, Chunk, ChunkType, PollConfig, RetryPolicy};
use std::time::Duration;

async fn poll() -> nexrad::data::Result<()> {
    let config = PollConfig::new("KDMX").with_download_policy(
        RetryPolicy::default_download().with_initial_delay(Duration::from_millis(500)),
    );
    let stream = chunk_stream(config, |d| tokio::time::sleep(d));
    let mut stream = std::pin::pin!(stream);

    let mut chunks: Vec<Chunk> = Vec::new();
    while let Some(downloaded) = stream.next().await {
        let downloaded = downloaded?;
        let is_end = downloaded.identifier.chunk_type() == ChunkType::End;
        chunks.push(downloaded.chunk);
        if is_end {
            let scan = assemble_volume(std::mem::take(&mut chunks))?;
            println!("volume with {} sweeps", scan.sweeps().len());
        }
    }
    Ok(())
}
```

On `wasm32`, pass a timer future instead of tokio's, for example `|d| gloo_timers::future::sleep(d)`.

### Render a reflectivity sweep to PNG

0.x had no rendering library; the `render` example in `nexrad` 0.0.3 rasterised `Message31` gates by hand into a PPM
file. Use the 1.0 code below.

**1.0.0-rc.4**

```rust
use nexrad::model::data::Product;
use nexrad::render::{nws_reflectivity_scale, render_radials, RenderOptions};

fn main() -> nexrad::Result<()> {
    let scan = nexrad::load_file("KDMX20220305_232324_V06")?;
    let sweep = scan.sweeps().first().expect("at least one sweep");

    let options = RenderOptions::new(1024, 1024);
    let image = render_radials(sweep.radials(), Product::Reflectivity, &nws_reflectivity_scale(), &options)?;
    image
        .save("reflectivity.png")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}

// Or, with a field: render_sweep(&nexrad::extract_field(sweep, Product::Reflectivity).unwrap(),
//                                &default_color_scale(Product::Reflectivity), &options)?.save("reflectivity.png")?;
```

**1.0**

```rust
use nexrad::model::data::Product;
use nexrad::render::{default_scale, render_sweep, RenderOptions};

fn main() -> nexrad::Result<()> {
    let scan = nexrad::load_file("KDMX20220305_232324_V06")?;
    let sweep = scan
        .sweeps()
        .first()
        .ok_or_else(|| std::io::Error::other("no sweeps"))?;

    let field = sweep
        .field(Product::Reflectivity)
        .ok_or_else(|| std::io::Error::other("no reflectivity"))?;

    let scale = default_scale(Product::Reflectivity); // a ColorScale (the NWS reflectivity scale)
    let options = RenderOptions::new(1024, 1024); // u32 dimensions
    let result = render_sweep(&field, &scale, &options)?;
    result.save("reflectivity.png")?; // Error::Image(ImageError) on failure

    // Polar-only metadata is optional in 1.0:
    if let Some(px) = result.metadata().km_to_pixel_distance(100.0) {
        println!("100 km is {px} pixels");
    }
    Ok(())
}
```

`default_scale(product)` returns the standard scale for each product as a `ColorScale`; the individual scale functions
(`nws_reflectivity_scale()`, `velocity_scale()`, ...) still return a `DiscreteColorScale`, which converts with
`.into()`. To keep rendering an arbitrary radial slice, build the field with
`SweepField::from_radials(radials, product)`.

## Appendix: complete change tables

These tables list every removed, renamed or re-typed public item between each baseline and 1.0, grouped by type. Added
items that replace nothing are in the CHANGELOGs.

### nexrad-model 1.0.0-rc.2 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.2 | 1.0 |
|---|---|
| `BinaryData<T>` (and its impls) | Not public |
| `data::VCPNumber` (variants unchanged) | `data::VcpNumber`, `#[non_exhaustive]` |
| `data::CFPStatus` (variants unchanged) | `data::CfpStatus` |
| `data::CFPMomentData` (`from_fixed_point`, `iter`, `values`, `DataMoment` impl) | `data::MomentData` with `Product::ClutterFilterPower` |
| `data::CFPMomentValue::{Status, Value}` | `data::MomentValue::{CfpStatus, Value}` |
| `data::DataMoment` trait (`data_word_size`, `first_gate_range`, `first_gate_range_km`, `gate_count`, `gate_interval`, `gate_interval_km`, `offset`, `raw_gate_values`, `raw_values`, `scale`) | Inherent methods on `MomentData` |
| `data::ChannelConfiguration::SZ2Phase` | `Sz2Phase` |
| `data::WaveformType::{CS, CDW, CDWO, B, SPP}` | `{ContiguousSurveillance, ContiguousDopplerWithAmbiguityResolution, ContiguousDopplerWithoutAmbiguityResolution, Batch, StaggeredPulsePair}` |
| `data::RadialStatus::ElevationStartVCPFinal` | `ElevationStartVcpFinal` |
| `data::Product` | `#[non_exhaustive]` |
| `data::Product::moment_data(&Radial)`, `cfp_moment_data(&Radial)` | `Radial::moment(Product)` |
| `data::MomentData::from_fixed_point(u16, u16, u16, u8, f32, f32, Vec<u8>) -> Self` | `from_fixed_point(Product, usize, u16, u16, u8, f32, f32, Vec<u8>) -> Result<Self>` |
| `data::MomentData::gate_count() -> u16` | `-> usize` |
| `data::Radial::new(i64, u16, f32, f32, RadialStatus, u8, f32, Option<MomentData> × 6, Option<CFPMomentData>)` | `Radial::builder(u16, f32, u8, f32)` → `RadialBuilder`, `build() -> Result<Radial>` |
| `data::Sweep::new(u8, Vec<Radial>) -> Self` | `-> Result<Self>` |
| `data::Radial::clutter_filter_power() -> Option<&CFPMomentData>` | `-> Option<&MomentData>` |
| `data::Radial::{azimuth, azimuth_spacing, elevation_angle}() -> uom::si::f32::Angle` | `-> uom::si::f64::Angle` |
| `data::Sweep::elevation_angle() -> Option<uom::si::f32::Angle>` | `-> Option<uom::si::f64::Angle>` |
| `data::Scan::coverage_pattern_number() -> VCPNumber` | `-> VcpNumber` |
| `data::VcpNumber::{Precipitation12, ClearAirLongPulse31, ClearAirShortPulse32, ClearAir35, PrecipitationMpda112, PrecipitationSz2_212, GeneralSurveillance215}` | `{Vcp12, Vcp31, Vcp32, Vcp35, Vcp112, Vcp212, Vcp215}`; adds `Vcp11`, `Vcp21`, `Vcp121`, `Vcp211`, `Vcp221` |
| `data::Radial` (no PRF context) | Adds `nyquist_velocity_mps()`, `unambiguous_range_km() -> Option<f32>` (with `uom`, `nyquist_velocity()`, `unambiguous_range()`); `RadialBuilder::{nyquist_velocity_mps, unambiguous_range_km}` |
| `data::Scan` | Adds `elevation_cut(&Sweep) -> Option<&ElevationCut>`, `sweeps_with_waveform(WaveformType) -> impl Iterator<Item = &Sweep>` |
| `data::ElevationCut::new(f64, ChannelConfiguration, WaveformType, ..)` (22 arguments) | `ElevationCut::builder(f32, WaveformType, ChannelConfiguration)` → `ElevationCutBuilder` |
| `data::ElevationCut::elevation_angle_degrees() -> f64` | `-> f32` |
| `data::VolumeCoveragePattern::new(u16, ..)` (14 arguments) | `VolumeCoveragePattern::builder(VcpNumber)` → `VolumeCoveragePatternBuilder` |
| `data::VolumeCoveragePattern::pattern_number() -> VCPNumber` | `-> VcpNumber` |
| `data::SweepField::new_empty(label, unit, f32, Vec<f32>, f32, f64, f64, usize)` | `SweepField::builder(label, unit)` → `SweepFieldBuilder`, `build() -> Result<SweepField>` |
| `data::SweepField::value_at_polar(f32, f64)` | `value_at_polar(f64, f64)` |
| `data::SweepField::elevation_degrees()` | `elevation_angle_degrees()` |
| `data::SweepField::azimuths() -> &[f32]` | `azimuth_angles_degrees() -> &[f32]` |
| `data::SweepField::radial_status_mut(usize) -> &mut [GateStatus]` | `radial_statuses_mut(usize) -> Option<&mut [GateStatus]>` |
| `data::SweepField::radial_values(usize) -> &[f32]`, `radial_values_mut(usize) -> &mut [f32]`, `radial_statuses(usize) -> &[GateStatus]` | `-> Option<&[f32]>`, `-> Option<&mut [f32]>`, `-> Option<&[GateStatus]>` |
| `data::SweepField::get(usize, usize) -> (f32, GateStatus)` | `-> Option<(f32, GateStatus)>` |
| `data::SweepField::set(usize, usize, f32, GateStatus)` | `-> Result<()>` |
| `data::CartesianField::new(label, unit, GeoExtent, usize, usize) -> Self` | `-> Result<Self>` |
| `data::CartesianField::get(usize, usize) -> (f32, GateStatus)`, `set(..)` | `get -> Option<(f32, GateStatus)>`, `set -> Result<()>` |
| `data::VerticalField::new(label, unit, (f64, f64), (f64, f64), usize, usize) -> Self` | `-> Result<Self>` |
| `data::VerticalField::get(usize, usize) -> (f32, GateStatus)`, `set(..)` | `get -> Option<(f32, GateStatus)>`, `set -> Result<()>` |
| `data::GateStatus::{Valid, BelowThreshold, RangeFolded, NoData}` | Adds `Filtered`; still exhaustive |
| `impl From<&data::MomentValue> for data::GateStatus` | `impl From<data::MomentValue> for data::GateStatus` |
| `data::Radial::collection_timestamp() -> i64` | `collection_timestamp_millis() -> i64` |
| `data::VolumeCoveragePattern::doppler_velocity_resolution() -> f32` | `doppler_velocity_resolution_mps() -> f32` |
| `data::VolumeCoveragePattern::{sails_cuts, mrle_cuts}() -> u8` | `{sails_cut_count, mrle_cut_count}() -> u8` |
| `geo::PolarPoint::{azimuth_degrees, elevation_degrees}: f32` | `{azimuth_angle_degrees, elevation_angle_degrees}: f64` |
| `geo::RadarCoordinateSystem::gate_location(f32, f32, usize, f64, f64)` | `gate_location(f64, f64, usize, f64, f64)` |
| `geo::RadarCoordinateSystem::geo_to_polar(GeoPoint, f32)` (flat-earth range) | `geo_to_polar(GeoPoint, f64)`, the exact inverse of `polar_to_geo`; ranges change |
| `geo::RadarCoordinateSystem` | Adds `beam_height_meters(f64, f64) -> f64`, `ground_range_km(f64, f64) -> f64` |
| `meta::Site::new([u8; 4], f32, f32, i16, u16)` | `Site::builder([u8; 4], f64, f64)` → `SiteBuilder`, `build() -> Result<Site>` |
| `meta::Site::identifier() -> &[u8; 4]` | `-> &str` |
| `meta::Site::identifier_string() -> String` | Removed; `identifier()` |
| `meta::Site::{latitude, longitude}() -> f32` | `-> f64` |
| `meta::Site::height_meters() -> i16` | `site_elevation_meters() -> i16` |
| `meta::Site::tower_height_meters() -> u16` | `feedhorn_height_meters() -> u16` |
| `meta::Site::{height, tower_height}() -> uom::si::f32::Length` | `{site_elevation, feedhorn_height}() -> uom::si::f64::Length` |
| `meta::registry::SiteEntry` | `#[non_exhaustive]` |
| `meta::registry::SiteEntry::{latitude, longitude}: f32` | `f64` |
| `meta::registry::SiteEntry::elevation_meters: i16` | `site_elevation_meters: i16` |
| `meta::registry::nearest_site(f32, f32)` | `nearest_site(f64, f64)` |
| `meta::registry::SiteEntry::to_site() -> Site` | `-> Result<Site>` |
| `result::Error` | `#[non_exhaustive]`, `Clone`, `PartialEq`; new `InvalidGeometry { parameter, reason }`, `InvalidMomentData { parameter, reason }`, `MomentDataLength { expected, actual }`, `InvalidSite { parameter, reason }`, `IndexOutOfBounds { row, column, rows, columns }` |
| `result::Error::ElevationMismatch` (unit variant) | `ElevationMismatch { expected: u8, actual: u8 }` |
| serde: `data::MomentData` keys `first_gate_range`, `gate_interval`, `values` (a sequence of integers) | `first_gate_range_meters`, `gate_interval_meters`, `raw_values` (a byte string) |
| serde: `data::{SweepField, CartesianField, VerticalField}` key `status` | `statuses` |
| serde: `data::CfpStatus` variants in snake_case | PascalCase variant names |
| serde: `Deserialize` for `data::{SweepField, CartesianField, VerticalField, MomentData, Radial, Sweep}` and `meta::Site` (no validation) | Validates as the constructors and builders do |

</details>

### nexrad-decode 1.0.0-rc.3 to 1.0

<details>
<summary>Show the table</summary>

All paths are under `nexrad_decode::messages` unless they start with `result` or `summarize`.

**Types and enum variants**

| 1.0.0-rc.3 | 1.0 |
|---|---|
| `MessageType::RDADigitalRadarData`, `RDAStatusData`, `RDAPerformanceMaintenanceData`, `RDAConsoleMessage`, `RDAVolumeCoveragePattern`, `RDAControlCommands`, `RDALoopBackTest`, `RDAClutterFilterBypassMap`, `RDAClutterFilterMap`, `RDAAdaptationData`, `RDAPRFData`, `RDALogData`, `RDADigitalRadarDataGenericFormat` | `Rda...` (`RdaPrfData` for `RDAPRFData`) |
| `MessageType::RPGVolumeCoveragePattern`, `RPGClutterCensorZones`, `RPGRequestForData`, `RPGConsoleMessage`, `RPGLoopBackTest` | `Rpg...` |
| `MessageType::Spare1` (14), `ReservedFAARMSOnly1..5` (16, 17, 24, 25, 26), `Reserved1..5` (20–23, 29) | `MessageType::Reserved(u8)` holding the type number |
| `MessageType` with `#[repr(u8)]` discriminants | No discriminants; `number()`, `from_number(u8)`; `#[non_exhaustive]` |
| `MessageContents::RDAStatusData`, `RDAControlCommands`, `RDAPRFData`, `RDALogData`, `RDAAdaptationData` | `RdaStatusData`, `RdaControlCommands`, `RdaPrfData`, `RdaLogData`, `RdaAdaptationData`; `#[non_exhaustive]` |
| `RedundantChannel::ORDASingleChannel`, `ORDARedundantChannel1`, `ORDARedundantChannel2` | `OrdaSingleChannel`, `OrdaRedundantChannel1`, `OrdaRedundantChannel2`; `#[non_exhaustive]` |
| `MessageHeader` (`#[repr(C)]`, public fields, zerocopy traits) | `MessageHeader<'a>` with accessors; `parse`, `SIZE`, `into_owned` |
| `MessageHeaders::Single(&'a MessageHeader)`, `Multiple(Vec<&'a MessageHeader>)` | `Single(MessageHeader<'a>)`, `Multiple(Vec<MessageHeader<'a>>)` |
| `digital_radar_data::VolumeCoveragePattern::{VCP12, VCP31, VCP35, VCP112, VCP212, VCP215, Unknown}` | `VcpNumber::{Vcp11, Vcp12, Vcp21, Vcp31, Vcp32, Vcp35, Vcp112, Vcp121, Vcp211, Vcp212, Vcp215, Vcp221, Unknown}` |
| `digital_radar_data::DataMomentPointer`, `DataMomentPointerType`, `DataMomentGenericPointerType` | Removed |
| `digital_radar_data::CompressionIndicator::CompressedBZIP2`, `CompressedZLIB` | `CompressedBzip2`, `CompressedZlib` |
| `digital_radar_data::ProcessingStatus::RxRNoise`, `CBT` | `RxrNoise`, `Cbt` |
| `digital_radar_data::RadialStatus::ElevationStartVCPFinal` | `ElevationStartVcpFinal` |
| `digital_radar_data::DataBlockName` | `#[non_exhaustive]` |
| `rda_status_data::RDAStatus`, `RDABuildNumber`, `RMSControlStatus` | `RdaStatus`, `RdaBuildNumber`, `RmsControlStatus` |
| `rda_status_data::RMSControlStatus::{NonRMS, RMSInControl, RDAInControl}` | `RmsControlStatus::{NonRms, RmsInControl, RdaInControl}` |
| `rda_status_data::CommandAcknowledgement::RemoteVCPReceived` | `RemoteVcpReceived` |
| `rda_status_data::TransitionPowerSourceStatus::OK` | `Ok` |
| `rda_status_data::VolumeCoveragePatternNumber` (`number() -> i16`, `local()`, `remote()`) | `VcpSelection` (`number() -> VcpNumber`, `source() -> VcpSource`) |
| `volume_coverage_pattern::ChannelConfiguration::SZ2Phase` | `Sz2Phase` |
| `volume_coverage_pattern::WaveformType::{CS, CDW, CDWO, B, SPP}` | `{ContiguousSurveillance, ContiguousDopplerWithAmbiguityResolution, ContiguousDopplerWithoutAmbiguityResolution, Batch, StaggeredPulsePair}` |
| `clutter_censor_zones::Message::regions() -> &[raw Region]` (public `Integer2`/`Code2` fields) | `-> &[clutter_censor_zones::Region<'a>]` with accessors and `OperatorSelectCode` |
| `summarize` module (`messages`, `MessageSummary`, `MessageGroupSummary`, `VCPInfo`, `VCPElevationInfo`, `RDAStatusInfo`) | Removed |
| `result::Error::Io(std::io::Error)`, `From<std::io::Error>` | Removed |
| `result::Error` | `#[non_exhaustive]`; new `MissingCoveragePattern` and (feature `model`) `Model(nexrad_model::Error)` |

**Accessors**

| Type | 1.0.0-rc.3 | 1.0 |
|---|---|---|
| `MessageHeader` | fields `segment_size`, `redundant_channel`, `message_type`, `sequence_number`, `date` + `time`, `segment_count`, `segment_number` | `segment_size_raw()`, `redundant_channel_raw()`, `message_type_raw()`, `sequence_number()`, `date_time()`, `segment_count()`, `segment_number()` |
| `MessageHeader` | `rda_redundant_channel()` | `redundant_channel()` |
| `MessageHeader` | (none) | `segment_size_bytes()` |
| `digital_radar_data::Header` | `azimuth_angle_raw()`, `elevation_angle_raw()`, `radial_length_raw()` | `azimuth_angle_degrees()`, `elevation_angle_degrees()`, `radial_length_bytes()` |
| `digital_radar_data::Header` | `azimuth_resolution_spacing_type() -> AzimuthResolutionSpacing` | `azimuth_resolution_spacing() -> AzimuthResolutionSpacing` |
| `digital_radar_data::Header` | `azimuth_resolution_spacing() -> Angle` (uom) | `azimuth_resolution_spacing_degrees() -> f32` |
| `digital_radar_data::VolumeDataBlock` | `lrtup_raw()`, `latitude_raw()`, `longitude_raw()`, `site_height_raw()`, `tower_height_raw()` | `lrtup_bytes()`, `latitude_degrees()`, `longitude_degrees()`, `site_height_meters()`, `tower_height_meters()` |
| `digital_radar_data::VolumeDataBlock` | `calibration_constant()`, `system_differential_reflectivity()` | `calibration_constant_db()`, `system_differential_reflectivity_db()` |
| `digital_radar_data::VolumeDataBlock` | `horizontal_shv_tx_power_raw()`, `vertical_shv_tx_power_raw()` | `horizontal_shv_tx_power_kw()`, `vertical_shv_tx_power_kw()` |
| `digital_radar_data::VolumeDataBlock` | `initial_system_differential_phase_raw()` | `initial_system_differential_phase_degrees()` |
| `digital_radar_data::VolumeDataBlock` | `volume_coverage_pattern() -> VolumeCoveragePattern`, `volume_coverage_pattern_number()` | `volume_coverage_pattern() -> VcpNumber`, `volume_coverage_pattern_raw()` |
| `digital_radar_data::RadialDataBlock` | `lrtup_raw()`, `horizontal_channel_noise_level()`, `vertical_channel_noise_level()` | `lrtup_bytes()`, `horizontal_channel_noise_level_dbm()`, `vertical_channel_noise_level_dbm()` |
| `digital_radar_data::RadialDataBlock` | `horizontal_channel_calibration_constant()`, `vertical_channel_calibration_constant()` | `horizontal_channel_calibration_constant_db()`, `vertical_channel_calibration_constant_db()` |
| `digital_radar_data::RadialDataBlock` | (none) | `unambiguous_range_km()`, `nyquist_velocity_mps()` |
| `digital_radar_data::ElevationDataBlock` | `lrtup_raw()`, `calibration_constant()`, `atmos()` | `lrtup_bytes()`, `calibration_constant_db()`, `atmos_db_per_km()` |
| `digital_radar_data::GenericDataBlockHeader` | (none) | `tover_db()`, `snr_threshold_db()`, `moment_size_bytes()` |
| `digital_radar_data::GenericDataBlock` | `moment_data()`, `into_moment_data()`, `cfp_moment_data()`, `into_cfp_moment_data()` | `DataBlock<'a, GenericDataBlock<'a>>::moment_data() -> Result<MomentData>`, `into_moment_data()` |
| `digital_radar_data::Message` | `site() -> Option<Site>` | `site() -> Result<Option<Site>>` |
| `digital_radar_data_legacy::Message` | `azimuth_angle()`, `elevation_angle()`, `calibration_constant()` | `azimuth_angle_degrees()`, `elevation_angle_degrees()`, `calibration_constant_db()` |
| `digital_radar_data_legacy::Message` | `surveillance_first_gate_range()`, `doppler_first_gate_range()`, `surveillance_gate_interval()`, `doppler_gate_interval()` | `..._meters()` |
| `digital_radar_data_legacy::Message` | `doppler_velocity_resolution()` | `doppler_velocity_resolution_mps()` |
| `digital_radar_data_legacy::Message` | `elevation_number() -> u16`, `radial_status() -> u16`, `vcp_number() -> u16` | `elevation_number() -> u8`, `radial_status() -> RadialStatus` + `radial_status_raw()`, `vcp_number() -> VcpNumber` + `vcp_number_raw()` |
| `volume_coverage_pattern::Header` | `message_size()`, `doppler_velocity_resolution()` | `message_size_halfwords()`, `doppler_velocity_resolution_mps()` |
| `volume_coverage_pattern::ElevationDataBlock` | `elevation_angle()`, `azimuth_rate()`, `sector_1_edge_angle()`, `sector_2_edge_angle()`, `sector_3_edge_angle()`, `ebc_angle()` | `elevation_angle_degrees()`, `azimuth_rate_degrees_per_second()`, `sector_N_edge_angle_degrees()`, `ebc_angle_degrees()` |
| `volume_coverage_pattern::ElevationDataBlock` | `reflectivity_threshold()`, `velocity_threshold()`, `spectrum_width_threshold()`, `differential_reflectivity_threshold()`, `differential_phase_threshold()`, `correlation_coefficient_threshold()` | `..._threshold_db()` |
| `clutter_filter_map::RangeZone` | `raw_op_code()`, `end_range() -> u16`, `end_range_uom()` | `op_code_raw()`, `end_range_km() -> u16`, `end_range()` (uom) |
| `request_for_data::Message` | `raw_data_request_type()` | `data_request_type_raw()` |
| `rda_status_data::Message` | `raw_rda_status()`, `raw_operability_status()`, `raw_control_status()`, `raw_auxiliary_power_generator_state()`, `raw_horizontal_reflectivity_calibration_correction()`, `raw_data_transmission_enabled()`, `raw_volume_coverage_pattern()`, `raw_rda_control_authorization()`, `raw_operational_mode()`, `raw_super_resolution_status()`, `raw_clutter_mitigation_decision_status()`, `raw_rda_scan_and_data_flags()`, `raw_rda_alarm_summary()`, `raw_command_acknowledgement()`, `raw_channel_control_status()`, `raw_spot_blanking_status()`, `raw_vertical_reflectivity_calibration_correction()`, `raw_transition_power_source_status()`, `raw_rms_control_status()`, `raw_performance_check_status()`, `raw_alarm_codes()`, `raw_signal_processor_options()` | The same stems with a `_raw` suffix (`rda_status_raw()`, ..., `signal_processor_options_raw()`) |
| `rda_status_data::Message` | `raw_rda_build_number()` | `build_number_raw()` |
| `rda_status_data::Message` | `average_transmitter_power()`, `horizontal_reflectivity_calibration_correction()` | `average_transmitter_power_watts()`, `horizontal_reflectivity_calibration_correction_db()` |
| `rda_status_data::Message` | (none) | `vertical_reflectivity_calibration_correction_db()` |
| `console_message::Message` | `message_size()` | `message_size_bytes()` |
| `loopback_test::Message` | `message_size()` | `message_size_halfwords()` |
| `rda_log_data::Message` | `compressed_size()`, `decompressed_size()` | `compressed_size_bytes()`, `decompressed_size_bytes()` |
| `rda_adaptation_data::Message` | `raw_data()` | `data_raw()` |
| `clutter_censor_zones::Region` | fields `start_range`, `stop_range`, `start_azimuth`, `stop_azimuth`, `elevation_segment_number`, `operator_select_code` | `start_range_km()`, `stop_range_km()`, `start_azimuth_degrees()`, `stop_azimuth_degrees()`, `elevation_segment_number()`, `operator_select_code()` / `operator_select_code_raw()` |

The `rda_adaptation_data` (type 18) and `performance_maintenance_data` (type 3) accessors keep their ICD mnemonic names.

</details>

### nexrad-data 1.0.0-rc.7 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.7 | 1.0 |
|---|---|
| `result::aws` module, `result::aws::AWSError` | `result::AwsError` (also `nexrad_data::AwsError`), `#[non_exhaustive]` |
| `AWSError::S3GetObjectRequest(reqwest::Error)`, `S3ListObjects(reqwest::Error)`, `S3Streaming(reqwest::Error)` | Same names holding `Box<dyn Error + Send + Sync>` |
| `AWSError::PollingAsync` | Removed |
| `result::Error::AWS` | `Error::Aws` |
| `result::Error::Decompression(bzip2::Error)`, `From<bzip2::Error>` | Removed |
| `result::Error` | `#[non_exhaustive]` |
| `volume::split_compressed_records(&[u8])` | Removed; `File::records()` |
| `volume::Header::{extension_number, icao_of_radar, tape_filename}() -> Option<String>` | `-> Option<&str>` |
| `volume::File::header() -> Option<&Header>`; `#[repr(C)] volume::Header` with `zerocopy::{FromBytes, FromZeros, Immutable, KnownLayout, TryFromBytes}` | `header() -> Option<Header>`; `Header` without zerocopy traits, `Header::SIZE` |
| `volume::Record::decompress<'b>(&self) -> Result<Record<'b>>` | `decompress(&self) -> Result<Record<'static>>` |
| `aws::archive::list_files(&str, &NaiveDate)` | `list_files(&str, NaiveDate)` |
| `aws::archive::download_file(Identifier)` | `download_file(&Identifier)` |
| `aws::realtime::Chunk<'a>`, `Chunk::IntermediateOrEnd(Record<'a>)` | `Chunk`, `Chunk::IntermediateOrEnd(Record<'static>)` |
| `aws::realtime::DownloadedChunk::chunk: Chunk<'static>` | `chunk: Chunk`; struct is `#[non_exhaustive]`, `DownloadedChunk::new` |
| `aws::realtime::ChunkIteratorInit`, `LatestVolumeResult` | `#[non_exhaustive]` |
| `aws::realtime::assemble_volume<'a>(impl IntoIterator<Item = Chunk<'a>>)` | `assemble_volume(impl IntoIterator<Item = Chunk>)` |
| `aws::realtime::chunk_stream(PollConfig)` | `chunk_stream(PollConfig, S) where S: Fn(Duration) -> F, F: Future<Output = ()>` |
| `aws::realtime::download_chunk<'a>(&str, &ChunkIdentifier) -> Result<(ChunkIdentifier, Chunk<'a>)>` | `download_chunk(&str, &ChunkIdentifier) -> Result<DownloadedChunk>` |
| `aws::realtime::ChunkIdentifier::new(String, ..)`, `from_name(String, VolumeIndex, String, ..)` | `new(impl Into<String>, ..)`, `from_name(impl Into<String>, VolumeIndex, impl Into<String>, ..)` |
| `aws::realtime::ChunkIdentifier::next_chunk(&ElevationChunkMapper)`, `NextChunk::{Sequence, Volume}` | Removed |
| `aws::realtime::ChunkIterator::start_with_policies(&str, RetryPolicy, RetryPolicy)` | `start_with(PollConfig)` |
| `aws::realtime::ChunkIterator::from_chunk(&str, ChunkIdentifier, RetryPolicy, RetryPolicy)` | `from_chunk(PollConfig, ChunkIdentifier)` |
| `aws::realtime::ChunkIterator::{download_policy, discovery_policy}()` | `config().download_policy()`, `config().discovery_policy()` |
| `aws::realtime::ChunkIterator::time_until_next() -> Option<chrono::Duration>` | `-> Option<std::time::Duration>` |
| `aws::realtime::ChunkIterator::project_remaining_scan() -> Option<ScanTimingProjection>` | `projected_volume_end_time() -> Option<DateTime<Utc>>` |
| `aws::realtime::ChunkIterator::{timing_stats, timing_stats_mut, elevation_mapper, chunk_metadata, all_chunk_metadata}()` | Removed |
| `aws::realtime::PollConfig::{site, download_policy, discovery_policy}` fields | Getters of the same names |
| `aws::realtime::RetryPolicy::{max_attempts, initial_delay, backoff_multiplier, max_delay}` fields | Getters of the same names (`std::time::Duration`) |
| `aws::realtime::RetryPolicy::with_initial_delay(chrono::Duration)`, `with_max_delay(chrono::Duration)` | Take `std::time::Duration` |
| `aws::realtime::{ChunkTimingModel, ChunkTimingStats, ChunkCharacteristics, ElevationChunkMapper, ChunkMetadata, ChunkProjection, ScanTimingProjection, RetryState}` | Removed |
| `aws::realtime::{estimate_chunk_availability_time, estimate_chunk_processing_time, project_scan_timing, project_full_scan_timing}` | Removed |
| (new) | `volume::File::coverage_pattern_message() -> Result<volume_coverage_pattern::Message<'static>>` |
| `aws::realtime::ChunkIterator::try_next()`, `chunk_stream(..)` at volume rollover: newest chunk of the next volume | The next volume's Start chunk, then each later chunk in sequence order |

</details>

### nexrad-process 1.0.0-rc.1 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.1 | 1.0 |
|---|---|
| `ScanProcessor` trait (`name`, `process_scan`) | Removed |
| `ScanDerivedProduct::compute(&self, &Scan, &[SweepField], &RadarCoordinateSystem, &GeoExtent, (usize, usize))` | `compute(&self, &[SweepField], &RadarCoordinateSystem, &GeoExtent, (usize, usize))` (also `CompositeReflectivity::compute`) |
| `filter::ThresholdFilter::{min, max}` fields | `ThresholdFilter::new(Option<f32>, Option<f32>) -> Result<Self>`, `min()`, `max()` |
| `filter::MedianFilter::{azimuth_kernel, range_kernel}` fields | `MedianFilter::new(usize, usize) -> Result<Self>`, getters |
| `filter::GaussianSmooth::{sigma_azimuth, sigma_range}` fields | `GaussianSmooth::new(f32, f32) -> Result<Self>`, getters |
| `derived::VerticalCrossSection::new(f32, f64, f64, usize, usize)` | `new(f64, f64, f64, usize, usize)` |
| `derived::VerticalCrossSection::compute(&self, &[SweepField])` | `compute(&self, &[SweepField], &RadarCoordinateSystem)` |
| `detection::StormCell::elevation_degrees() -> f32` | `elevation_angle_degrees() -> f32` |
| `detection::StormCell::max_reflectivity_azimuth_degrees() -> f32` | `max_reflectivity_azimuth_angle_degrees() -> f32` |
| `filter::{ThresholdFilter, CorrelationCoefficientFilter}` mask gates as `(0.0, GateStatus::NoData)` | Mask gates as `(value, GateStatus::Filtered)` |
| `result::Error` | `#[non_exhaustive]`; new `Model(nexrad_model::Error)` |

</details>

### nexrad-render 1.0.0-rc.4 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.4 | 1.0 |
|---|---|
| `render_radials(&[Radial], Product, &DiscreteColorScale, &RenderOptions) -> Result<RgbaImage>` | Removed; `render_sweep` |
| `render_radials_default(&[Radial], Product, &RenderOptions) -> Result<RgbaImage>` | Removed; `render_sweep` with `default_scale` |
| `default_color_scale(Product) -> ColorScale` | Removed; `default_scale` |
| `default_scale(Product) -> DiscreteColorScale` | `default_scale(Product) -> ColorScale` |
| `ColorLookupTable::from_scale(&DiscreteColorScale, f32, f32, usize)`, `from_color_scale(&ColorScale, f32, f32, usize)` | `ColorLookupTable::new(&ColorScale, f32, f32, usize)` |
| `RenderOptions::new(usize, usize)`, `size() -> (usize, usize)` | `new(u32, u32)`, `size() -> (u32, u32)` |
| `RenderResult::new(RgbaImage, RenderMetadata)` | Crate-private |
| `RenderResult::query_polar(&SweepField, f32, f64)` | `query_polar(&SweepField, f64, f64)` |
| `RenderMetadata::center_pixel() -> (f64, f64)` | `-> Option<(f64, f64)>` |
| `RenderMetadata::{pixels_per_km, max_range_km}() -> f64` | `-> Option<f64>` |
| `RenderMetadata::{pixel_distance_to_km, km_to_pixel_distance}(f64) -> f64` | `-> Option<f64>` |
| `RenderMetadata::polar_to_pixel(f32, f64) -> (f64, f64)` | `polar_to_pixel(f64, f64) -> Option<(f64, f64)>` |
| `RenderMetadata::elevation_degrees() -> Option<f32>` | `elevation_angle_degrees() -> Option<f64>` |
| `PointQuery` | `#[non_exhaustive]` |
| `result::Error::{NoRadials, ImageSave(String), ProductNotFound}` | `EmptyField`, `Image(image::ImageError)`, removed |
| `result::Error` | `#[non_exhaustive]` |

</details>

### nexrad 1.0.0-rc.4 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.4 | 1.0 |
|---|---|
| `extract_field`, `extract_fields`, `extract_first_field` | `Sweep::field`, `Scan::fields`, `Scan::first_field` |
| `coordinate_system`, `coordinate_system_required` | `Scan::coordinate_system` |
| `sites`, `site`, `nearest_site(f32, f32)` | `model::meta::registry::{sites, site_by_id, nearest_site(f64, f64)}` |
| `download(Identifier)` | `download(&Identifier)` |
| `stream(&str)` | `stream(&str, S) where S: Fn(Duration) -> F, F: Future<Output = ()>` |
| `Error::MissingSiteMetadata` | Removed |
| `Error::NoDataAvailable::date: String` | `date: chrono::NaiveDate`; variant requires `aws` |
| `Error` | `#[non_exhaustive]` |
| Features `aws-polling`, `tokio` | Removed; use `aws` |

</details>

### 0.x baselines

<details>
<summary>Show the notes</summary>

The [Upgrading from 0.x](#upgrading-from-0x) tables cover every public entry point of `nexrad` 0.0.3,
`nexrad-model` 0.1.0, `nexrad-decode` 0.1.2 and `nexrad-data` 0.2.0. Beyond them, the 0.x diffs differ from the
release-candidate diffs above only in:

- `nexrad` 0.0.3: every item of the `model` module (`DataFile`, `Message31`, `Message31Header`, `MessageHeader`,
  `DataMoment`, `GenericData`, `DataBlockHeader`, `VolumeData`, `ElevationData`, `RadialData`, `VolumeHeaderRecord`)
  is removed; see [nexrad 0.0.3](#nexrad-003) for their successors.
- `nexrad-decode` 0.1.2: the message structs gained a lifetime (`Message<'a>`, `Header<'a>`, ...) and lost their public
  fields; accessors replace them with the names in the
  [nexrad-decode table](#nexrad-decode-100-rc3-to-10). Legacy digital radar data,
  performance/maintenance, console, control command, clutter map, censor zone, request, loopback, adaptation, PRF and
  log messages are decoded into typed `MessageContents` variants instead of `Other`.
- `nexrad-data` 0.2.0: the realtime timing and iterator API (`ChunkIterator`, `chunk_stream`, `PollConfig`,
  `RetryPolicy`, `DownloadedChunk`, `assemble_volume`) is new, and `File::compressed`/`decompress` handle gzip-wrapped
  volumes.

</details>

### How these tables were produced

The tables come from [`cargo public-api`](https://github.com/cargo-public-api/cargo-public-api) diffs of each crate
against its published baseline, with trait-implementation noise removed, cross-checked against the CHANGELOGs and the
pull requests they cite. To see the full diff for the version you are on, run this from a checkout of the repository
(it needs a nightly toolchain for rustdoc JSON; the repository pins one in `xtask/src/public_api.rs`):

```sh
cargo install cargo-public-api
RUSTUP_TOOLCHAIN=nightly cargo public-api -p nexrad-data --all-features --simplified diff 1.0.0-rc.5
```

Replace the crate and version with yours. The output has "Removed items", "Changed items" and "Added items" sections;
items whose path or signature changed appear as a removal plus an addition.
