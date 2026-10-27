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

From an earlier release candidate, follow the release-candidate section; the compiler points to the small extra
changes. `nexrad 0.1.0-rc1` contained only an `Error` enum and is not covered.

The minimum supported Rust version is **1.88** for every crate. The release candidates declared 1.75, which was already
incorrect: they used items that entered the prelude in 1.80, and their dependencies needed 1.88.

### How to use this guide

The `nexrad` facade re-exports the five sub-crates as `nexrad::model`, `nexrad::decode`, `nexrad::data`,
`nexrad::process` and `nexrad::render`, and adds the I/O entry points (`load`, `download`, `list_scans`, `stream` and
their variants) and a unified `nexrad::Error`.

- **Facade users** read the facade section, then the sections of the sub-crates whose types they touch (usually
  `nexrad::model`, often `nexrad::render`).
- **Sub-crate users** read the section for each crate; they are in dependency order.
- **0.x users** read [Upgrading from 0.x](#upgrading-from-0x) first, then the release-candidate sections.

The per-crate sections explain semantics, errors and features; the [appendix](#appendix-complete-change-tables) maps
every renamed, removed or re-typed item to its replacement and pull request. The CHANGELOGs
([`nexrad`](../nexrad/CHANGELOG.md), [`nexrad-model`](../nexrad-model/CHANGELOG.md),
[`nexrad-decode`](../nexrad-decode/CHANGELOG.md), [`nexrad-data`](../nexrad-data/CHANGELOG.md),
[`nexrad-process`](../nexrad-process/CHANGELOG.md), [`nexrad-render`](../nexrad-render/CHANGELOG.md)) list every
change, and [architecture.md](architecture.md) states the rules behind them: [layering](architecture.md#layering-rules),
[features](architecture.md#feature-architecture), [errors](architecture.md#error-architecture),
[naming](architecture.md#naming-conventions) and [enum stability](architecture.md#enum-stability).

Snippets use the facade paths (`nexrad::model::...`); with a sub-crate, write `nexrad_model` for `nexrad::model` and so
on. Paths below the crate root are identical.

### Changes that apply to every crate

**1. `Error` and `Result` at every crate root** (#158), plus `nexrad_data::AwsError` with `aws`. The `result` modules
remain.

```rust
use nexrad_data::{AwsError, Error, Result}; // nexrad_data::result::Error still works
```

**2. Every `Error` enum is `#[non_exhaustive]`** (#155), as are `nexrad_data::AwsError`,
`nexrad_model::data::Product`, `nexrad_decode::messages::{MessageContents, MessageType, RedundantChannel}`,
`nexrad_decode::messages::digital_radar_data::DataBlockName` and `nexrad_render::Projection`. A `match` outside the
defining crate needs a wildcard arm. `VcpNumber` (in both crates) is a `u16` newtype with associated constants rather
than an enum (#205); a `match` on it needs a wildcard arm too. The deliberately exhaustive enums are listed under
[Enum Stability](architecture.md#enum-stability).

```rust
match product {
    Product::Reflectivity => "dBZ",
    // ...
    _ => product.unit(),
}
```

The ICD code enums in `nexrad-decode`'s message modules (every one with an `Unknown(n)` or `Reserved(n)` variant, such
as `rda_status_data::RdaStatus` and `digital_radar_data::CompressionIndicator`) and `nexrad_render::{ColorScale,
Interpolation}` are non-exhaustive too (#207). An `Unknown(n)` or `Reserved(n)` value may become a named variant in a
minor release, so compare `number()` or the `*_raw()` accessor where you need the same answer across releases.

**3. Acronyms are written as words** (#160), following Rust's RFC 430, and the terse waveform variants got descriptive
names. The renames change the serialized form; see [Serialized data](#serialized-data).

```rust
// 1.0.0-rc.N: VCPNumber, RDAStatus, AWSError, CFPStatus, SZ2Phase, WaveformType::CS
// 1.0:        VcpNumber, RdaStatus, AwsError, CfpStatus, Sz2Phase, WaveformType::ContiguousSurveillance
```

**4. Optional dependencies are no longer features** (#156). With `dep:` syntax, `nexrad-model`, `tokio`, `futures`,
`reqwest`, `rayon`, `xml` and `once_cell` are not feature names; features are named for what they enable (`model`,
`aws`, `parallel`, `serde`, `chrono`). No crate depends on an async runtime; see
[No Async Runtime Dependency](architecture.md#no-async-runtime-dependency).

```toml
nexrad-data = { version = "1.0", default-features = false, features = ["model", "parallel"] } # was ["nexrad-model", "rayon"]
```

**5. Wrapping error variants display only their layer** (#195). `nexrad::Error::{Model, Decode, Data, Render, Process,
Io}`, `nexrad_data::Error::{Io, Decode, Model}`, `nexrad_decode::Error::Model`, `nexrad_process::Error::Model` and
`nexrad_render::Error::Encode` display a layer name (`decode error`) and return the wrapped error from `source()`;
`nexrad_data::Error::Aws` displays the `AwsError` itself. Walk the source chain to print the whole cause:

```rust
let mut message = err.to_string();
let mut source = std::error::Error::source(&err);
while let Some(cause) = source {
    message = format!("{message}: {cause}");
    source = cause.source();
}
```

**6. The `uom` feature and its accessors are removed** (#202). `uom` has no 1.0 release, and its quantity types in
public signatures would tie the major version of these crates to `uom`'s. `nexrad-model`, `nexrad-decode` and `nexrad`
no longer have a `uom` feature; drop it from your `features` lists. Every removed accessor has a sibling that returns a
plain number with the unit in its name, so switch to that, or build the quantity yourself if you want `uom` types.
Unit-typed accessors may come back later as a separate crate or an extension trait.

```rust
// 1.0.0-rc.N with features = ["uom"]
let elevation = radial.elevation_angle();          // uom::si::f64::Angle
let range = radial_data_block.unambiguous_range(); // uom::si::f64::Length

// 1.0
let elevation = radial.elevation_angle_degrees();          // f32
let range = radial_data_block.unambiguous_range_km();      // f32
// With your own `uom` dependency:
let elevation = uom::si::f64::Angle::new::<uom::si::angle::degree>(f64::from(elevation));
```

| Removed (`uom` feature) | Use instead |
|---|---|
| `nexrad_model::data::Radial::{azimuth, azimuth_spacing, elevation_angle}()` | `azimuth_angle_degrees()`, `azimuth_spacing_degrees()`, `elevation_angle_degrees()` (`f32`) |
| `nexrad_model::data::Radial::{nyquist_velocity, unambiguous_range}()` | `nyquist_velocity_mps()`, `unambiguous_range_km()` (`Option<f32>`) |
| `nexrad_model::data::Sweep::elevation_angle()` | `elevation_angle_degrees()` (`Option<f32>`) |
| `nexrad_model::data::MomentData::{first_gate_range, gate_interval}()` | `first_gate_range_km()`, `gate_interval_km()` (`f64`), or `first_gate_range_meters()`, `gate_interval_meters()` (`u16`) |
| `nexrad_model::meta::Site::{site_elevation, feedhorn_height}()` | `site_elevation_meters()` (`i16`), `feedhorn_height_meters()` (`u16`) |
| `nexrad_decode::messages::MessageHeader::{message_size, segment_size}()` | `message_size_bytes()` (`u32`), `segment_size_bytes()` (`Option<u32>`) |
| `digital_radar_data::Header::{azimuth_angle, elevation_angle}()` | `azimuth_angle_degrees()`, `elevation_angle_degrees()` (`f32`) |
| `digital_radar_data::Header::azimuth_indexing_mode()` | `azimuth_indexing_mode_degrees()` (`Option<f32>`) |
| `digital_radar_data::Header::radial_length()` | `radial_length_bytes()` (`u16`) |
| `digital_radar_data::{VolumeDataBlock, ElevationDataBlock, RadialDataBlock}::lrtup()` | `lrtup_bytes()` (`u16`) |
| `digital_radar_data::VolumeDataBlock::{latitude, longitude}()` | `latitude_degrees()`, `longitude_degrees()` (`f32`) |
| `digital_radar_data::VolumeDataBlock::{site_height, tower_height}()` | `site_height_meters()` (`i16`), `tower_height_meters()` (`u16`) |
| `digital_radar_data::VolumeDataBlock::{horizontal_shv_tx_power, vertical_shv_tx_power}()` | `horizontal_shv_tx_power_kw()`, `vertical_shv_tx_power_kw()` (`f32`) |
| `digital_radar_data::VolumeDataBlock::initial_system_differential_phase()` | `initial_system_differential_phase_degrees()` (`f32`) |
| `digital_radar_data::RadialDataBlock::{unambiguous_range, nyquist_velocity}()` | `unambiguous_range_km()`, `nyquist_velocity_mps()` (`f32`) |
| `digital_radar_data::GenericDataBlockHeader::{data_moment_range, data_moment_range_sample_interval}()` | `data_moment_range_km()`, `data_moment_range_sample_interval_km()` (`f64`) |
| `digital_radar_data::GenericDataBlockHeader::moment_size()` | `moment_size_bytes()` (`u32`) |
| `digital_radar_data_legacy::Message::nyquist_velocity()` | `nyquist_velocity_mps()` (`f32`) |
| `clutter_filter_map::RangeZone::end_range()` | `end_range_km()` (`u16`) |

## Upgrading from a release candidate

### nexrad-model from 1.0.0-rc.2

Renames and removals are in the [nexrad-model table](#nexrad-model-100-rc2-to-10). The `uom` feature is removed; see
[change 6](#changes-that-apply-to-every-crate).

**Builders, `Result` and `Option`.** Constructors with long argument lists became builders (#165) that take the
required values, have a setter per optional value and end with `.build()`. `Radial::builder` defaults to timestamp `0`,
spacing `1.0` and `IntermediateRadialData`, and `ElevationCut::builder` takes the waveform **before** the channel
configuration. Constructors with invariants return `Result`, and field indexing returns `Option` (#185):

```rust
// 1.0.0-rc.2
let radial = Radial::new(
    timestamp, azimuth_number, azimuth_angle_degrees, 0.5, RadialStatus::IntermediateRadialData,
    elevation_number, elevation_angle_degrees, Some(reflectivity), None, None, None, None, None, None,
);
let site = Site::new(*b"KDMX", 41.7312, -93.7229, 299, 34);
let sweep = Sweep::new(1, radials);
let (value, status) = field.get(azimuth_idx, gate_idx);

// 1.0
let radial = Radial::builder(azimuth_number, azimuth_angle_degrees, elevation_number, elevation_angle_degrees)
    .collection_timestamp_millis(timestamp)
    .azimuth_spacing_degrees(0.5)
    .moment(reflectivity) // stored under reflectivity.product()
    .build()?;
let site = Site::builder(*b"KDMX", 41.7312, -93.7229)
    .site_elevation_meters(299)
    .feedhorn_height_meters(34)
    .build()?;
let sweep = Sweep::new(1, radials)?;
let Some((value, status)) = field.get(azimuth_idx, gate_idx) else { return Ok(()) };
```

**Moment data, elevation cuts and coverage patterns** (#210). `MomentData::from_fixed_point` is replaced by a
builder with named setters; the gate count defaults to the number of gates in the raw values and is checked when set.
`ElevationCutBuilder::build` and `VolumeCoveragePatternBuilder::build` return `Result`:

```rust
// 1.0.0-rc.2
let moment = MomentData::from_fixed_point(3, 2125, 250, 8, 2.0, 66.0, vec![0, 1, 100]);
let cut = ElevationCut::new(0.5, ChannelConfiguration::ConstantPhase, WaveformType::CS, /* 19 more */);
let n = vcp.number_of_elevation_cuts();
let sweeps = Sweep::from_radials(radials);

// 1.0
let moment = MomentData::builder(Product::Reflectivity, vec![0, 1, 100])
    .first_gate_range_meters(2125)
    .gate_interval_meters(250)
    .scale(2.0)
    .offset(66.0)
    .build()?;
let cut = ElevationCut::builder(0.5, WaveformType::ContiguousSurveillance, ChannelConfiguration::ConstantPhase)
    .reflectivity_snr_threshold_db(2.0)
    .build()?;
let vcp = VolumeCoveragePattern::builder(VcpNumber::VCP_212).elevation_cut(cut).build()?;
let n = vcp.elevation_cut_count();
let sweeps = Sweep::split_by_elevation_number(radials);
```

**Errors** (#185). `ElevationMismatch` is a struct variant (match `ElevationMismatch { expected, actual }` or
`ElevationMismatch { .. }`), and `Error` implements `Clone` and `PartialEq`. New variants, where `parameter` names the
offending argument:

- `InvalidGeometry { parameter, reason }` from `SweepFieldBuilder::build`, `RadialBuilder::build`,
  `CartesianField::new` and `VerticalField::new`;
- `InvalidMomentData { parameter, reason }` from `MomentDataBuilder::build` for an unsupported `data_word_size`, a
  `gate_count` above `u16::MAX`, or a non-finite `scale` or `offset`;
- `MomentDataLength { expected, actual }` (in bytes) when its `values` do not hold exactly `gate_count` gates;
- `InvalidCoveragePattern { parameter, reason }` from `ElevationCutBuilder::build` and
  `VolumeCoveragePatternBuilder::build` (#210);
- `InvalidSite { parameter, reason }` from `SiteBuilder::build` and `registry::SiteEntry::to_site`;
- `IndexOutOfBounds { row, column, rows, columns }` from the field `set` methods.

**Behaviour and type changes**

- **`VcpNumber` is a `u16` newtype** (#205): match on the constants (`VcpNumber::VCP_212`) where you matched a
  variant such as `VCPNumber::PrecipitationSz2_212`, build one with `VcpNumber::new(n)` or `n.into()` where you used
  `from_number(n)` or `Unknown(n)`, and read the number with `number()`, which now takes `self`. `description()` is
  removed, and `Display` prints only the number (`VCP 212`, not `VCP 212 (Precipitation, SZ-2)`). Values order by
  number. The serialized form is unchanged.
- **Unknown codes carry the code** (#205): `WaveformType`, `ChannelConfiguration` and `PulseWidth` have `Unknown(u8)`
  instead of a unit `Unknown`, like `RadialStatus`, and match `nexrad-decode`'s copies variant for variant. Match
  `Unknown(_)`. `Display` of an unknown value includes the code (`Unknown (7)`), and `RadialStatus` implements
  `Display`.
- **Coordinates and polar angles are `f64`** (#164): literals keep compiling; convert `f32` variables with
  `f64::from(x)`. The stored `*_degrees` angles on `Radial`, `Sweep`, `SweepField` and `ElevationCut` are `f32`
  (#182), so `Scan::sweeps_at_elevation` still takes `f32`.
- **Exhaustive enums gained variants**: `MomentValue::CfpStatus(CfpStatus)`, produced only for
  `Product::ClutterFilterPower` and mapped to `GateStatus::NoData`, and `GateStatus::Filtered` (#187), a gate whose
  valid value a processing step removed, keeping the value. Exhaustive matches need the new arms.
- **`GateStatus` is `#[non_exhaustive]`** (#211): its `Filtered` and `NoData` statuses are set by processing steps, and
  a later step may add one. Add a wildcard (`_`) arm to matches on it. `MomentValue` stays exhaustive.
- **`SiteEntry::state` is `Option<&'static str>`** (#211), so the registry can list sites outside the United States.
  Every current entry is `Some`; compare with `Some("OK")` or use `site.state.unwrap_or_default()` where you used the
  string. `Display` prints `KTLX (Oklahoma City, OK)` as before and omits the state when it is `None`.
- **Constructors validate** (#185). `SweepFieldBuilder::build` requires non-empty, finite, strictly ascending azimuths
  in `0.0..360.0`, a non-zero gate count, a finite positive gate interval and azimuth spacing, a finite non-negative
  first gate range and a finite elevation. `CartesianField::new` and `VerticalField::new` reject zero or overflowing
  dimensions and non-finite or unordered (`min >= max`) ranges, and a Cartesian extent must lie within valid latitudes
  and longitudes. `MomentDataBuilder::build` requires a word size of 8 or 16, exactly `gate_count` gates and a
  finite scale and offset. `RadialBuilder::build` requires a finite azimuth in `0.0..360.0`, a finite elevation and a
  positive spacing. `SiteBuilder::build` requires four ASCII alphanumeric identifier bytes and an in-range location.
  `ElevationCutBuilder::build` requires a finite elevation angle, azimuth rate and signal-to-noise thresholds, and
  `VolumeCoveragePatternBuilder::build` a finite, non-negative Doppler velocity resolution (#210).
- **Indexing is checked** (#185): `get(0, width + 3)` returns `None` instead of reading the next row, and `set` and
  the row accessors report out-of-range indices instead of panicking.
- **Unscaled moments decode their sentinels** (#185): for every product except clutter filter power, raw 0 and 1
  decode to `BelowThreshold` and `RangeFolded` even when `scale` is `0.0`, not to `Value(0.0)` and `Value(1.0)`.
- **`SweepField::from_radials` geometry** (#185): the elevation (the median) and azimuth spacing come from the radials
  carrying the product, not the sweep's first radial, and radials whose first gate range or gate interval differ from
  the first such radial are skipped instead of mis-registered.
- **`SweepField::value_at_polar` respects sector coverage** (#185): the azimuth is normalized into `0.0..360.0`, the
  lookup wraps across north only for a full circle, and azimuths more than half a spacing outside a sector return
  `None` instead of the far end's value.
- **`SweepField::product()`** records the field's product (`Some` from `from_radials`, copied by `new_like`, optional on
  the builder) (#170).
- **Site registry** (#148): added `KBHX`, `KDOX`, `KHDC`, `KLGX`, `PAPD`; removed `KLIX` (now `KHDC`), the invalid
  Alaska aliases `KABC`, `KACG`, `KAIH`, `KAKC` (use the `P...` identifiers) and a duplicate `KCCX`; corrected `KFDX`
  and `KPBZ`. `site_by_id("KLIX")` returns `None`, and `nearest_site` can return a different site near those locations.
  The registry is data: minor releases may add, correct or remove entries, including sites of other radar kinds such
  as TDWR, and `nearest_site` returns the nearest listed site (#210).
- **`geo_to_polar` returns `Option<PolarPoint>`** (#210): `None` when no beam at the elevation passes over the
  point (`elevation + central angle` of 90 degrees or more) or an input is not finite, where it returned a non-finite
  or negative range. Handle the `None` (`let Some(polar) = cs.geo_to_polar(point, elevation) else { continue };`).
- **`geo_to_polar` ranges change** (#188). It is the exact inverse of `polar_to_geo` under one model (spherical earth of
  radius 6371 km, 4/3 effective-radius refraction, antenna height above mean sea level) instead of
  `ground / cos(elevation)` without antenna height, so ranges grow by about 3.6 km at 300 km on a 19.5 degree tilt and
  by metres on low tilts. `polar_to_geo` moves by under a metre. `beam_height_meters` and `ground_range_km` expose the
  same geometry.
- **`registry::nearest_site` uses great-circle distance** (#188), which corrects results near the antimeridian.
  Registry sites have a feedhorn height of 0; build a `RadarCoordinateSystem` from `Scan::site()`.
- **Radials carry Nyquist velocity and unambiguous range** (#188), filled by the `nexrad-decode` conversions. Dealias
  with the per-radial value: Doppler PRF sectors vary by azimuth within a sweep.
- **Sweeps link to their elevation cut** (#188). Split cuts, SAILS, MRLE and MPDA repeat elevations, so `Scan::fields`,
  `first_field` and `sweeps_at_elevation` can return several sweeps per angle with reflectivity and velocity in
  different ones; select with `Scan::elevation_cut(&sweep)` or
  `Scan::sweeps_with_waveform(WaveformType::ContiguousSurveillance)`.
- **`Scan::coverage_pattern()` returns `Option<&VolumeCoveragePattern>`** (#208). It is `None` for a scan built
  without a volume coverage pattern message (type 5), such as a pre-2008 volume, and then `elevation_cut` returns
  `None` and `sweeps_with_waveform` yields nothing. `Scan::coverage_pattern_number()` is stored on the scan and always
  present; it can differ from the pattern's own `pattern_number()` (see nexrad-decode). `Scan::new` and
  `Scan::with_site` take the number and the optional pattern:

  ```rust
  // 1.0.0-rc.2
  let scan = Scan::with_site(site, vcp, sweeps);
  let cuts = scan.coverage_pattern().elevation_cuts();

  // 1.0
  let scan = Scan::with_site(site, vcp.pattern_number(), Some(vcp), sweeps);
  let cuts = scan.coverage_pattern().map(|vcp| vcp.elevation_cuts()).unwrap_or_default();
  ```
- **`Debug` output** prints the new names, `fnv1a: 0x...` instead of `sha256: "..."` for binary blobs, a `product` field
  on moments and `nyquist_velocity_mps`/`unambiguous_range_km` on `Radial`; update snapshot tests that capture it.

### nexrad-decode from 1.0.0-rc.3

Renames and removals are in the [nexrad-decode tables](#nexrad-decode-100-rc3-to-10).

**Features** (#156, #202). `nexrad-model` is now `model` (still default), and **`uom` is removed**: switch to the
unit-suffixed accessors (see [change 6](#changes-that-apply-to-every-crate)).

```toml
nexrad-decode = { version = "1.0", default-features = false, features = ["model"] } # was ["nexrad-model"] or ["nexrad-model", "uom"]
```

**Errors.** `Error` implements `Clone` and `PartialEq`. `Io` and `From<std::io::Error>` are gone (#168); nothing
produced them, so handle I/O errors where you do the I/O. Variants carry offsets into the slice you passed to the
decoder, including `InvalidDataBlockPointer`'s `position` (#190). New variants:

- `MissingCoveragePatternNumber` (feature `model`), from `ScanBuilder::build` when there was no type 5 message and no
  radial carried a non-zero coverage pattern number (#169, #208);
- `NegativeFirstGateRange { range_meters }` (feature `model`), from the legacy (type 1) conversion when the first gate
  is at a negative range and the gate interval is zero (#208);
- `Model(nexrad_model::Error)` (feature `model`) when decoded values fail model validation: a moment whose gate count
  does not match its data, an out-of-range radial azimuth, or an invalid site identifier or location (#185); it
  displays `decoded data is invalid for the model` (#195);
- `SegmentOutOfSequence { message_type, expected_segment, segment_count, offset }` for a missing or out-of-order
  segment (#190).

**Accessor naming** (#173, #191; see [Naming Conventions](architecture.md#naming-conventions) and "Accessor Naming" in
the crate docs). The stem is the ICD field name. A decoded plain number carries a unit suffix
(`azimuth_angle_degrees()`, `message_size_halfwords()`); `<stem>_raw()` is the wire encoding, only where it differs,
and replaces `raw_<stem>()`; a decoded enum or struct takes the bare stem (`radial_status()`); counts are
`<thing>_count()`; a byte string is `<stem>_raw()` beside `<stem>() -> Option<&str>`. One name changed meaning:
`digital_radar_data::Header::azimuth_resolution_spacing()` is now the `AzimuthResolutionSpacing` enum (the number is
`azimuth_resolution_spacing_degrees()`). `clutter_filter_map::RangeZone::end_range()` is now `end_range_km()`.

**`summarize` is removed** (#168); iterate instead:

```rust
for message in &messages {
    match message.contents() {
        MessageContents::DigitalRadarData(m) => {
            let h = m.header();
            println!("elevation {} azimuth {}", h.elevation_number(), h.azimuth_angle_degrees());
            if let Some(volume) = m.volume_data_block() {
                println!("{}", volume.inner().vcp_number()); // "VCP 212"
            }
        }
        MessageContents::RdaStatusData(s) => println!("{:?}", s.rda_status()),
        _ => {}
    }
}
```

**Scan assembly moved here** (#169): `nexrad_decode::model::ScanBuilder` (feature `model`) with `new`, `add_message`,
`add_messages`, `merge` and `build`.

**Behaviour and type changes**

- **Decoding is strict** (#190). `decode_messages` returns the error for the first message it cannot decode (truncated,
  bad data block pointer, missing segment) instead of logging and dropping it, or stopping
  with what it had. A tail shorter than a message header (28 bytes) ends decoding cleanly when every byte is zero. To
  keep the messages before a failure:

  ```rust
  let mut messages = Vec::new();
  let mut iter = nexrad_decode::iter_messages(data);
  for result in iter.by_ref() {
      match result {
          Ok(message) => messages.push(message),
          Err(error) => log::warn!("decoding stopped at offset {}: {error}", iter.offset()),
      }
  }
  ```

- **`MessageContents::Other { message_type, data }`** carries the header's type and the undecoded payload (#190), and
  `MessageContents::message_type()` names any contents' type. Each type 0 frame (an unused metadata record slot) is its
  own `Other`, and maps from a legacy (pre-ORDA) RDA decode as `Other`: the type 15 clutter filter map with its legacy
  notch width layout (#190) and the type 13 bypass map, which decoded as an empty map (#191).
- **Unrecognised data blocks do not fail the decode** (#207). A type 31 data moment block (type byte `'D'`) whose name
  this version does not know is parsed with the generic layout and kept in
  `digital_radar_data::Message::other_data_blocks()`; an unknown `'R'` block is skipped. Neither is part of `radial()`.
  `Error::UnknownDataBlockType` is now returned only for a block whose type byte is neither `'R'` nor `'D'`, and by
  `moment_data()` on one of the other data blocks.
- **`DataBlock` no longer derefs** (#190): reach the block through `inner()` or `into_inner()`
  (`m.volume_data_block().map(|b| b.inner().vcp_number())`); `moment_data()` and `into_moment_data()` stay on the
  `DataBlock`.
- **Multi-segment messages decode their full payload** (#190): type 13 bypass maps decode instead of being dropped,
  type 15 maps have 360 azimuth segments instead of 350, and type 18 fields past the first segment, such as
  `site_name()`, hold their real values.
- **VCP numbers** (#163, #188, #205): `VcpNumber` is a `u16` newtype with the same shape as
  `nexrad_model::data::VcpNumber`. Match on its constants (`VcpNumber::VCP_212`); build one with `VcpNumber::new(n)`
  or `n.into()`. It implements `Display` (`VCP 212`) and `Ord` by number, and with the new `serde` feature serializes
  as the bare number.
- **The VCP code enums match the model** (#205): `RadialStatus::VolumeScanStart` and `VolumeScanEnd` are `ScanStart`
  and `ScanEnd`, the model's names. `WaveformType`, `ChannelConfiguration`, `PulseWidth` and `RadialStatus` gain
  `from_number(u8)`, `number()` and `Display`, `WaveformType` gains `abbreviation()`, and the `From` conversions to the
  model (feature `model`) keep the code of an `Unknown` value and also run from the model back to decode.
- **Converted radials carry radar physics**: `digital_radar_data::Message::radial()`/`into_radial()` set the Nyquist
  velocity and unambiguous range from the radial data block, `None` when it is absent (#188); the legacy (type 1)
  conversion sets the unambiguous range (#188) and the Nyquist velocity, `None` when zero (#191), from its header.
- **Scans without a type 5 message build** (#208). `ScanBuilder::build` returns a scan whose `coverage_pattern()` is
  `None` instead of failing. The scan's `coverage_pattern_number()` is the type 5 message's number when it is non-zero,
  and otherwise the first non-zero number in a type 1 header or type 31 volume data block, so a 2005 volume whose type
  5 message is numbered 0 reports its radials' VCP 32. It fails with `MissingCoveragePatternNumber` only when neither
  source has a number.
- **Legacy Doppler gates moved** (#208). The legacy (type 1) conversion turned the signed range to the first gate into
  its absolute value, so the Doppler first gate at -375 m became +375 m and every velocity and spectrum width gate sat
  750 m too far out. Gates at a negative range are now dropped: those moments start at 125 m and have two fewer gates.
- **Fixed decodes**: negative VCP elevation and sector angles are negative instead of wrapped (359.8 for -0.2) (#146);
  spectrum width blocks are `DataBlockName::SpectrumWidth`, not `Unknown(*b"SW ")` (#173); the rc.3 `uom`
  `RadialDataBlock::unambiguous_range()` was 10× too large, and its replacement `unambiguous_range_km()` applies the ICD
  0.1 km scale (#173); legacy radials with an
  unknown status convert to `RadialStatus::Unknown(v)`, not `IntermediateRadialData` (#173).
- **Code enums have one fallback** (#191): a value the ICD does not list is `Unknown(<wire type>)`; match `Unknown(_)`
  where you matched a unit variant. The ICD's "unknown" transition power source code 4 is `Unknown(4)`.
- **`RdaBuildNumber` is a number** (#191): `from_raw(u16)`, `major()`, `minor()`, `raw()`, `Display` (`"24.0"`) and
  build ordering (`build >= RdaBuildNumber::from_raw(2000)`) replace the `Build12_0`..`Build24_0` variants.
- **VCP and azimuth indexing angles are `f32`** (#191), matching the model; `azimuth_rate_degrees_per_second()` stays
  `f64`.
- **Message type 29 is `MessageType::ModelData`** (#206), the Archive II ICD's Model Data message, where it was
  `Reserved(29)`; match `ModelData` where you matched `Reserved(29)`. Its contents still decode as `Other`.
- **`MessageType` ordering** (#166, #205): `Ord` compares `number()`, so `Reserved(14)` sorts between
  `RpgLoopbackTest` (12) and `ClutterFilterMap` (15) instead of after `RdaLogData`.
- **Lifetimes** (#166): `MessageHeaders::primary()`/`iter()` and `Message::header()` borrow from `self`, and
  `Message::contents()` returns `&MessageContents<'a>`. No public type implements zerocopy traits.

### nexrad-data from 1.0.0-rc.7

Renames and removals are in the [nexrad-data table](#nexrad-data-100-rc7-to-10).

**Features.** `aws-polling` is removed; streaming is part of the default `aws` (#161). `aws` no longer includes a TLS
backend: the new default feature `aws-rustls` adds rustls with the aws-lc-rs provider, which `aws` used to enable
(#204). With `default-features = false`, add `aws-rustls` next to `aws` on native targets, or requests to the HTTPS
endpoints fail at runtime; on `wasm32`, `aws` alone is enough. `nexrad-model` is now `model`, and
`futures`, `once_cell`, `rayon`, `reqwest`, `tokio` and `xml` are no longer features (#156). `nexrad-decode`'s default
features are no longer enabled through this crate.

```toml
nexrad-data = { version = "1.0" }                                                               # was features = ["aws-polling"]
nexrad-data = { version = "1.0", default-features = false, features = ["model", "parallel"] } # was ["nexrad-model", "rayon"]
nexrad-data = { version = "1.0", default-features = false, features = ["aws-rustls"] }       # was ["aws"]
```

**Errors.** `AwsError` moved to the crate root, and its HTTP variants box the error instead of holding a
`reqwest::Error` (#160, #171). `is_transient()` says whether a failure is worth retrying (#171); `HttpClient` reports a
failure to build the HTTP client, and none of the variants added in #192 is transient. `is_transient()` is true for
transport failures, timeouts and the HTTP statuses 408, 429 and 5xx only; 400, 403 and other 4xx statuses are terminal
(#206). The struct-like variants (`TruncatedRecord`, `InvalidRecordSize`, `InvalidIdentifier`, `InvalidVolumeIndex`,
`S3GetObject`) are `#[non_exhaustive]`, so add `..` to patterns that list their fields (#206). `Error` and `AwsError` no
longer implement `UnwindSafe`/`RefUnwindSafe`; wrap them in `AssertUnwindSafe` if you need that.

```rust
// 1.0.0-rc.7
match err {
    nexrad_data::result::Error::AWS(AWSError::S3GetObjectRequest(e)) => retry(e.is_timeout()),
    nexrad_data::result::Error::AWS(AWSError::S3ObjectNotFound) => missing(),
    other => fail(other),
}

// 1.0
match err {
    Error::Aws(aws) if aws.is_transient() => retry(),
    Error::Aws(AwsError::S3ObjectNotFound) => missing(),
    _ => fail(),
}
```

**Real-time API.** `aws::realtime` was pruned to its polling surface (#167): the timing model is crate-private and its
estimates are exposed through `ChunkIterator`, configuration goes through `PollConfig`, and `chunk_stream` takes your
runtime's sleep function (#161).

**Behaviour and type changes**

- **Decoding errors propagate** (#190): `Record::messages()`, `Record::radials()`, `File::scan()`,
  `File::coverage_pattern_message()` and `assemble_volume` fail with `Error::Decode` where they skipped. For a lenient
  decode of one record, pass `record.data()` to `nexrad_decode::iter_messages`.
- **`get_latest_volume`** and the new `get_latest_volume_from_hint` return a `#[non_exhaustive]` `LatestVolumeResult`
  whose `volume` is `Option<VolumeIndex>` (#167).
- **`VolumeIndex::new` is fallible** (#192) outside `1..=999`; when restoring one from storage (for example a hint),
  handle the error, for instance by falling back to `get_latest_volume`.
- **`list_chunks_in_volume` has no cap** (#149, #192): it reads every listing page and returns every chunk of the
  newest scan in a reused volume directory, sorted by sequence. Take what you need (`.first()`, `.last()`,
  `.iter().take(n)`); a cap of 1 used as an existence probe becomes `.is_empty()`.
- **Archive identifiers are validated** (#192): `Identifier::parse` accepts `SSSSYYYYMMDD_HHMMSS` (UTC) with optional
  `_Vnn`, `_MDM` and `.gz` suffixes. `list_files` and `list_files_with_sizes` return volume files only, skipping `_MDM`
  objects and unparseable names. Equality, ordering and hashing use the name only.
- **`ChunkIterator::try_next` makes one attempt** (#192) and never sleeps: `Ok(None)` means not available yet, so poll
  again on your own cadence (for example after `time_until_next()`). Request errors propagate (check
  `is_transient()`), and it no longer returns `ExpectedChunkNotFound`. `PollConfig`'s retry policies apply to
  `chunk_stream` only.
- **`chunk_stream` yields its startup chunks** (#192): the volume's Start chunk when joining mid-volume (that first
  volume is partial), then the latest chunk and those after it.
- **Volume rollover yields the Start chunk** (#183): `try_next` and `chunk_stream` enter a new volume at sequence 1
  (volume header and VCP) and continue in sequence order, instead of jumping to its newest chunk. `assemble_volume`
  consumers no longer hit `Error::MissingCoveragePattern` after the first volume; consumers that skipped ahead see the
  extra chunks. `volume::File::coverage_pattern_message()` reads a `Chunk::Start` file's VCP.
- **Real-time volume selection** (#149, #154): `get_latest_volume` no longer returns an older volume when the newest sits
  just after the wrap-around, and `ChunkIterator` waits instead of replaying an older scan left in the next volume
  directory.
- **`File::scan` and `File::coverage_pattern_message` decompress gzip-wrapped files** (#208): they accept a gzip-wrapped
  file, so the `File::new(bytes).decompress()?` step is needed only before `File::records()`, which still returns
  `Error::CompressedFile` for a gzip-wrapped file. `File::new` stores the bytes as given, and `decompress` is unchanged.
- **Volumes without a type 5 message scan** (#208): `File::scan` and `assemble_volume` return a scan with
  `coverage_pattern()` `None` (see nexrad-decode) for a legacy volume or chunks without a Start chunk, where they
  returned `Error::MissingCoveragePattern`. They fail only when no coverage pattern number is found, with
  `Error::Decode(MissingCoveragePatternNumber)`. `File::coverage_pattern_message()` still returns
  `Error::MissingCoveragePattern`.
- **`File::scan` includes legacy (type 1) radials** (#169), as `Record::radials()` does, and `assemble_volume` accepts
  them; legacy-only volumes yield populated sweeps.
- **A scan's `Site::identifier` comes from the type 31 message header** (#169), not the Archive II volume header ICAO;
  they agree for well-formed files, and a missing ICAO no longer produces `[0, 0, 0, 0]`.
- **`File::records()` returns `Error::TruncatedRecord`** instead of panicking on data shorter than the 24-byte volume
  header (#171).
- **Record decompression is idempotent** (#206): `Record::decompress` returns an uncompressed record unchanged, so drop
  any `compressed()` check or `UncompressedData` arm around it. `CompressedData` is `CompressedRecord`.
- **`Record::radials()` is all-or-nothing** (#206): one radial that fails to convert fails the record, and a legacy
  volume is one record. `Record::iter_radials()` yields one `Result<Radial>` per radar data message for lenient
  callers.
- **Real-time instants are `DateTime<Utc>`** (#206): `ChunkIdentifier::new` takes the date-time prefix as
  `DateTime<Utc>` (`naive.and_utc()`), and `date_time_prefix()` and `volume()` return values, so drop the `*` or `&`
  around them. `download_chunk(&id)` drops the site argument; the identifier carries it. `VolumeIndex::as_number()`
  is `number()`, matching the other `number()` accessors.
- **`ChunkType` is `#[non_exhaustive]`** (#206): add a wildcard arm. `list_chunks_in_volume` skips, with a `log`
  warning, keys that do not parse as chunk names, so one odd key no longer fails `get_latest_volume`, `ChunkIterator`
  or `chunk_stream`.
- **`ChunkIterator::from_chunk` iterates** (#206): its first `try_next` downloads the current volume's Start chunk to
  learn the VCP (not yielded), then returns the next chunk; if the Start chunk is gone or has no VCP, it continues with
  the next volume.
- **Native HTTP timeouts** (#206): requests fail with a transient transport error after 30 s without a connection, 60 s
  without response bytes, or 300 s in total. wasm32 builds use the browser's limits.

### nexrad-process from 1.0.0-rc.1

Renames and removals are in the [nexrad-process table](#nexrad-process-100-rc1-to-10).

**Errors.** `Error` derives `Clone` and `PartialEq`, and its variants carry `&'static str` fields instead of a `String`
(#193). `InvalidParameter` comes from the constructors rather than from `process`. New variants:
`Model(nexrad_model::Error)` when the model rejects an output, such as a `CompositeReflectivity::compute` extent
outside valid latitudes and longitudes (#185; it displays `model error`, #195); `GeometryMismatch { parameter }` for a
CC field whose geometry differs from the input; and `UnexpectedProduct { expected, actual }` for an input holding the
wrong product (#193).

**Constructors and builders** (#175, #193). Processors have private fields and validating constructors.
`CompositeReflectivity::compute` drops the `scan` argument and takes a `CartesianGrid`. The `ScanDerivedProduct` trait is
removed (#207): `compute` is an inherent method, so drop the import, build the product with
`CompositeReflectivity::new()` (the unit struct is non-exhaustive) and replace your own impls with inherent methods or a
trait of your own. `SweepProcessor` impls must be `Send + Sync`:

```rust
// 1.0.0-rc.1
let pipeline = SweepPipeline::new()
    .then(ThresholdFilter { min: Some(5.0), max: None })
    .then(MedianFilter { azimuth_kernel: 3, range_kernel: 3 })
    .then(GaussianSmooth { sigma_azimuth: 1.0, sigma_range: 1.0 });
let composite = CompositeReflectivity.compute(&scan, &fields, &coords, &extent, (512, 512))?;
let section = VerticalCrossSection::new(200.0, 230.0, 18000.0, 600, 300)?;

// 1.0
let pipeline = SweepPipeline::new()
    .then(ThresholdFilter::new(Some(5.0), None)?)
    .then(MedianFilter::new(3, 3)?)
    .then(GaussianSmooth::new(1.0, 1.0)?);
let grid = CartesianGrid::new(extent, 512, 512)?;
let composite = CompositeReflectivity::new().compute(&fields, &coords, &grid)?;
let section = VerticalCrossSection::builder(200.0)
    .max_range_km(230.0)
    .max_altitude_meters(18000.0)
    .width(600)
    .height(300)
    .build()?;
```

**Behaviour and type changes**

- **Parameters are validated** (#175, #193): `MedianFilter::new` rejects zero or even kernels, `GaussianSmooth::new`
  sigmas that are not finite and positive (NaN and infinity were accepted), and `ThresholdFilter::new` a NaN bound or
  `min > max`. Every floating-point parameter must be finite; `StormRelativeVelocity::new` and the
  `VerticalCrossSection` builder need a direction or azimuth in `0.0..360.0`, and `CorrelationCoefficientFilter::new` a
  threshold in `0.0..=1.0`.
- **Masked gates are `Filtered`** (#187): `ThresholdFilter` and `CorrelationCoefficientFilter` mark removed gates
  `GateStatus::Filtered` and keep their values instead of setting `NoData` with `0.0`; look for `Filtered` where you
  looked for `NoData` (checks for `Valid` are unaffected). Smoothing and storm-relative velocity pass them through;
  composite reflectivity, cross-sections and threshold region detection skip them.
- **Inputs are checked by product** (#193); build fields with `SweepField::from_radials` or set
  `SweepFieldBuilder::product`. `CompositeReflectivity::compute`, `VerticalCrossSection::compute` and
  `ThresholdRegionDetector::detect` need `Product::Reflectivity`, `CorrelationCoefficientFilter::new` a
  `Product::CorrelationCoefficient` field and `StormRelativeVelocity` a `Product::Velocity` input; its output is
  labelled "Storm-Relative Velocity" with no product, so it no longer passes as a velocity moment.
- **`VerticalCrossSection::compute` takes the coordinate system** (#188), for example `scan.coordinate_system()`: the
  vertical axis is altitude above mean sea level including the antenna height, and the horizontal axis is ground range.
- `CorrelationCoefficientFilter` requires the input and CC fields to share azimuth angles, first gate range and gate
  interval (`Error::GeometryMismatch`); gate counts may differ (#193).
- **`StormCellDetector` and `StormCell` are `ThresholdRegionDetector` and `ThresholdRegion`** (#207), named for what
  they compute: connected regions at or above one reflectivity threshold. Behaviour and methods are unchanged.
- `ThresholdRegion::centroid()` is weighted by linear reflectivity factor rather than dBZ, moving it toward the most intense
  gates (#193).
- `MedianFilter`, `GaussianSmooth` and `ThresholdRegionDetector` no longer wrap a sector field's azimuths across north (#193).
- The processors implement `Debug`, `Clone` and `PartialEq`, and `CompositeReflectivity` also `Copy` and `Default`
  (#175).

### nexrad-render from 1.0.0-rc.4

Renames and removals are in the [nexrad-render table](#nexrad-render-100-rc4-to-10).

**Errors** (#174, #194). `NoRadials` is `EmptyField`, also returned for Cartesian or vertical fields with no rows or
columns, and `InvalidDimensions` is also returned for a zero width or height. `ImageSave(String)` is replaced by
`Io(std::io::Error)` and `Encode(Box<dyn Error + Send + Sync>)`, which displays `error encoding image` (#195, #203).
`ColorLookupTable::new` and an invalid `RenderOptions::with_value_range` return `InvalidRange { min, max }` or
`InvalidTableSize { size }`, and `render_sweep` with an extent returns `InvalidExtent` or `MissingCoordinateSystem`.
Because the boxed encoder error is not `UnwindSafe`, neither is `Error`.

**Pixels instead of `image` types** (#203). `RenderResult` owns its pixels as 8-bit RGBA, row-major with no row
padding; no `image` type is in the API. `image()` and `into_image()` become `pixels()` and `into_pixels()` (with
`width()` and `height()`), and `save(path)` becomes `save_png(path)`, which writes PNG whatever the extension;
`write_png(writer)` encodes into memory. The `RgbaImage` re-export is removed. To keep using `image`, enable the opt-in
`image` feature (`nexrad-render = { version = "1.0", features = ["image"] }`, or the facade's `image` feature) and call
`into_rgba_image()`; that feature tracks `image`'s 0.x releases and is outside the semver promise. For formats other
than PNG, pass the pixels to the image library of your choice:

```rust
// 1.0.0-rc.4
result.save("radar.png")?;
let image: RgbaImage = result.into_image();

// 1.0
let (width, height) = (result.width(), result.height());
result.save_png("radar.png")?;
let rgba: Vec<u8> = result.into_pixels();
```

**Render through a `SweepField`** (#174):

```rust
// 1.0.0-rc.4
let image = render_radials(sweep.radials(), Product::Reflectivity, &nws_reflectivity_scale(), &options)?;

// 1.0
let field = sweep
    .field(Product::Reflectivity)
    .ok_or_else(|| std::io::Error::other("no reflectivity"))?;
let pixels = render_sweep(&field, &default_scale(Product::Reflectivity), &options)?.into_pixels();
```

**Behaviour and type changes**

- **Metadata is projection-aware** (#174). `RenderMetadata::projection()` is `Polar`, `Geographic` or `Vertical`.
  Polar-only accessors and the point queries return `None` for `render_cartesian` and `render_vertical` results, which
  reported `pixels_per_km = 0` and divided by zero in `pixel_to_polar`; for `render_cartesian`, `pixel_to_geo` and
  `geo_to_pixel` map linearly through the extent. The variants are `#[non_exhaustive]`
  (`Projection::Polar { center_pixel, .. }`), with the fields `elevation_angle_degrees` (`Polar`) and
  `altitude_range_meters` (`Vertical`) (#182).
- **Image dimensions are `u32`** (#174); `RenderOptions::native_for` saturates at `u32::MAX`.
- **`ColorLookupTable::new` validates** (#194): finite `min < max` (`min == max` mapped every value to NaN and
  `min > max` silently reversed the table) and a `size` of at least 2.
- **Scale constructors validate** (#210): `DiscreteColorScale::new` and `ContinuousColorScale::new` return `Result`
  and reject an empty list (which drew every value black) or a value that is not finite with
  `Error::InvalidColorScale`. Levels and stops are still sorted, so any order is accepted. `nws_reflectivity_scale()`
  is `reflectivity_scale()`.
- **Colour range** (#194): every renderer uses `RenderOptions::value_range()`, else the product range
  (`Product::value_range`), else the field's valid-value range, else `-32..95`. `render_sweep` used the field's own data
  range, so colours of a sweep with a known product are now stable across sweeps; pin a range with
  `with_value_range(min, max)`.
- **Discrete colours are exact** (#194): values on a threshold (20 dBZ, 30 dBZ, ...) no longer take the colour of the
  level below; `entries()` still samples the scale at evenly spaced values.
- **`Color::to_rgba8` rounds and clamps** (#194), so some built-in colours shift by one step (`0.6627` is 169, not 168).
- **`RenderOptions::with_extent` renders into the extent** (#194): `render_sweep` draws onto a latitude/longitude grid
  covering exactly that extent (`Projection::Geographic`) and needs `with_coord_system`; it used to only copy the
  extent into the metadata. `render_cartesian` and `render_vertical` ignore it.
- **Non-valid gates** are drawn as the background unless `RenderOptions::with_status_color(status, color)` gives them a
  colour, such as purple for `GateStatus::RangeFolded` (#194).
- **PNG only** (#194, #203): `save_png` and `write_png` encode PNG; `save` no longer infers a format from the file
  extension. Encode other formats from `pixels()` with your own image library.
- **Queries on a sector sweep** return `None` beyond the covered azimuths instead of the far end's value (#185).
  `EmptyField` is unreachable for Cartesian and vertical fields built with `new`, which rejects zero dimensions.

### nexrad from 1.0.0-rc.4

Renames and removals are in the [nexrad table](#nexrad-100-rc4-to-10).

**Features** (#156, #161, #204). `aws-polling` and `tokio` are removed; `stream` is part of `aws`. Sub-crates are
depended on without default features, and each facade feature enables what it needs: `decode` implies `model`, `data`
implies `decode`, `model` and `chrono`, `parallel` implies `data`, `aws` implies `data`, and `aws-rustls` implies `aws`.
A minimal local-file build (`features = ["data"]`) no longer compiles reqwest. The `uom` feature is removed
(#202); see [change 6](#changes-that-apply-to-every-crate).

- The default is `data`, `aws`, `aws-rustls` and `render` instead of `full`. Add `process`, `serde` or `parallel`
  if you relied on the default for them, or ask for `full`.
- `wasm` is removed. Every feature except `parallel` builds for `wasm32-unknown-unknown`; list the ones you use.
- `aws` no longer includes a TLS backend; `aws-rustls` adds rustls with the aws-lc-rs provider, as `aws` did before.
  On native targets, a build with `aws` and without `aws-rustls` (or a TLS feature of `reqwest` enabled elsewhere)
  fails at runtime on the HTTPS endpoints. On `wasm32`, `aws` alone is enough.

```toml
nexrad = { version = "1.0", default-features = false, features = ["aws-rustls"] } # was ["aws-polling"] or ["aws"]
nexrad = { version = "1.0", default-features = false, features = [               # was ["wasm"]
    "data", "render", "process", "aws", "serde", "chrono",
] }
nexrad = { version = "1.0", features = ["process"] }                              # was the default (`full`)
```

**Errors are flattened by layer** (#176): decode, model and I/O errors raised inside `nexrad-data`, including items
from `stream` and `stream_scans`, become `Error::Decode`, `Error::Model` and `Error::Io`; other data errors stay
`Error::Data`. `data error: error decoding NEXRAD data` becomes `decode error`, with the message in `source()` (#195).

```rust
match nexrad::load_file(path) {
    Ok(scan) => use_scan(scan),
    Err(nexrad::Error::Io(e)) => unreadable(e),
    Err(nexrad::Error::Decode(e)) => malformed(e), // was Error::Data(data::result::Error::Decode(e))
    Err(nexrad::Error::Data(e)) => archive_error(e),
    Err(e) => other(e),
}
```

**I/O entry points** (#195). `download_latest(site)` downloads the newest archived volume; for the last volume of a
date, `list_scans(site, date)` and `download` the last identifier. `download_at` returns the volume that started at
most 15 minutes before `datetime`, searching the previous UTC date when `datetime` precedes the date's first volume,
and `Error::NoScanAt` otherwise. `download_at` takes a `DateTime<Utc>` (#206); convert a `NaiveDateTime` you
know to be UTC with `.and_utc()`.

### Serialized data

**Model data serialized with serde by a release candidate does not deserialize with 1.0. Regenerate it from the Archive
II source files.** There is no compatibility shim. The incompatible changes are:

| Type | Change | PR |
|---|---|---|
| `WaveformType` | Variants serialize as `"ContiguousSurveillance"`, `"ContiguousDopplerWithAmbiguityResolution"`, `"ContiguousDopplerWithoutAmbiguityResolution"`, `"Batch"`, `"StaggeredPulsePair"` instead of `"CS"`, `"CDW"`, `"CDWO"`, `"B"`, `"SPP"` | #160 |
| `ChannelConfiguration` | `"Sz2Phase"` instead of `"SZ2Phase"` | #160 |
| `RadialStatus` | `"ElevationStartVcpFinal"` instead of `"ElevationStartVCPFinal"` | #160 |
| `WaveformType`, `ChannelConfiguration`, `PulseWidth` | An unknown value is `{"Unknown": n}` instead of `"Unknown"` | #205 |
| `Scan` | New `coverage_pattern_number` field (the VCP number); `coverage_pattern` is nullable and defaults to null when absent | #208 |
| `MomentData` | A struct with a `product` field next to the encoding fields, for every moment on a `Radial` including `clutter_filter_power`; it was `serde(transparent)` over an inner block | #170 |
| `SweepField` | Gains a `product` field | #170 |
| `Site` | `latitude` and `longitude` are `f64`; self-describing formats such as JSON still read old values, fixed-width binary formats such as bincode do not | #164 |
| `Site` | `height_meters` and `tower_height_meters` are `site_elevation_meters` and `feedhorn_height_meters` | #182 |
| `Radial` | `collection_timestamp` is `collection_timestamp_millis` | #182 |
| `Radial` | Gains `nyquist_velocity_mps` and `unambiguous_range_km` (`Option<f32>`); JSON reads data without them as `None`, bincode does not | #188 |
| `SweepField` | `elevation_degrees` and `azimuths` are `elevation_angle_degrees` and `azimuth_angles_degrees` | #182 |
| `VolumeCoveragePattern` | `doppler_velocity_resolution`, `sails_cuts` and `mrle_cuts` are `doppler_velocity_resolution_mps`, `sails_cut_count` and `mrle_cut_count` | #182 |
| `ElevationCut` | `elevation_angle_degrees` is `f32` (was `f64`) | #182 |
| `PolarPoint` | `azimuth_degrees` and `elevation_degrees` are `azimuth_angle_degrees` and `elevation_angle_degrees` | #182 |
| `MomentData` | `first_gate_range`, `gate_interval` and `values` are `first_gate_range_meters`, `gate_interval_meters` and `raw_bytes` | #189 |
| `MomentData` | `raw_bytes` is a byte string (`serialize_bytes`); JSON and other formats without a bytes type still use an array of integers, MessagePack and CBOR write a binary string | #189 |
| `SweepField`, `CartesianField`, `VerticalField` | `status` is `statuses` | #189 |
| `Site` | `identifier` is the four-character string (`"KTLX"`) instead of four byte values (`[75, 84, 76, 88]`); only the string deserializes | #189 |
| `CfpStatus` | Variants are PascalCase (`"FilterNotApplied"`, `{"Reserved": n}`) instead of snake_case (`"filter_not_applied"`) | #189 |
| `ElevationCut` | `reflectivity_threshold_db`, `velocity_threshold_db`, `spectrum_width_threshold_db`, `differential_reflectivity_threshold_db`, `differential_phase_threshold_db` and `correlation_coefficient_threshold_db` are `*_snr_threshold_db` | #210 |
| `ElevationCut`, `VolumeCoveragePattern` | Deserialization validates as the builders do: finite angles, azimuth rate and thresholds, and a finite, non-negative Doppler velocity resolution | #210 |
| `SweepField`, `CartesianField`, `VerticalField`, `MomentData`, `Radial`, `Site`, `Sweep` | Deserialization validates as the constructors do: one `values` and `statuses` entry per cell, exactly `gate_count` gates of `data_word_size` bits in `raw_bytes`, each `Radial` moment under its own product's key, in-range `Site` coordinates, and one `elevation_number` across a `Sweep`'s radials | #189 |

Unchanged: `VcpNumber` serializes as its `u16` number, and the field names of `Scan` and `Sweep` are the same. From
1.0, serialized field names equal accessor names, enum variants serialize as their PascalCase names, and the
representation is stable within 1.x; `nexrad-model/tests/snapshots/` shows it for a real scan.

Data serialized by `nexrad-model` 0.1.0 differs further (`Scan::coverage_pattern_number` instead of
`coverage_pattern`, `Site::height_meters`, `RadialStatus::VolumeScanStart`/`VolumeScanEnd`, a `MomentData` of only
scale, offset and values); regenerate it too. `Debug` output also changed and was never a stable format.

## Upgrading from 0.x

Read this section, then the [release-candidate section](#upgrading-from-a-release-candidate) for each crate you use:
every release-candidate change also applies to you.

### The crate split

`nexrad` 0.0.3 was a single crate; in 1.0 its responsibilities live in five crates under the `nexrad` facade.
`nexrad-model` 0.1.0, `nexrad-decode` 0.1.2 and `nexrad-data` 0.2.0 came after the split and already have the 1.0
shape.

The model replaces raw messages. 0.0.3's `DataFile::elevation_scans()` was a `HashMap<u8, Vec<Message31>>` of type 31
messages keyed by elevation number, and you applied scale and offset to raw gate bytes yourself. `nexrad::load` returns
a `Scan` of ordered `sweeps()` of `Radial`s whose `MomentData::iter()` yields decoded `MomentValue`s (`Value(f32)`,
`BelowThreshold`, `RangeFolded`, `CfpStatus`), and `Sweep::field(product)` gives the dense azimuth × gate `SweepField`
that processing and rendering consume. `nexrad_decode::messages` still offers protocol-level access and decodes many
more message types, including legacy type 1 radials, which `File::scan` uses alongside type 31. Downloads use plain
HTTPS to the public NOAA buckets instead of the `aws-sdk-s3` client and its credentials, region configuration and tokio
requirement, so they work on any async runtime and build for `wasm32`. Real-time data is a stream: the channel-based
`poll_chunks` became `chunk_stream(config, sleep)`, a `Stream` of `DownloadedChunk`s with retry policies,
`assemble_volume` turns a volume's chunks into a `Scan`, and the facade wraps both as `nexrad::stream(site, sleep)` and
`nexrad::stream_scans(site, sleep)`.

| 0.x | 1.0 |
|---|---|
| **`nexrad` 0.0.3 modules** | |
| `nexrad::download` (feature `download`, `aws-sdk-s3`) | `nexrad_data::aws::archive`, or `nexrad::list_scans` / `download` / `download_latest` / `download_at` (feature `aws`) |
| `nexrad::decompress` | `nexrad_data::volume` (`Record::decompress`), applied automatically by `File::scan` and `nexrad::load` |
| `nexrad::decode` | `nexrad_decode::messages` (protocol level) or `nexrad::load` / `nexrad_data::volume::File::scan` (model level) |
| `nexrad::model` | `nexrad_decode::messages` (wire-format structs) and `nexrad_model::data` (`Scan`, `Sweep`, `Radial`, `MomentData`) |
| `nexrad::file` | `nexrad_data::aws::archive::Identifier`, `nexrad_data::volume::{File, Record}` |
| `nexrad::result` | `nexrad::Error` and each sub-crate's `Error` |
| (the `render` example) | `nexrad_render`, a library crate |
| (none) | `nexrad_process`: filters and derived products |
| **Features** | |
| `nexrad` 0.0.3 `download` | `nexrad` `aws` (or `nexrad-data` `aws`) |
| `nexrad-data` 0.2.0 `decode` | Removed: decoding is always available |
| `nexrad-data` 0.2.0 `nexrad-model` | `model` |
| `nexrad-data` 0.2.0 `aws` (pulled in tokio) | `aws` (no tokio; real-time streaming included) |
| `nexrad-decode` 0.1.2 `nexrad-model` | `model` (default) |
| `nexrad-decode` 0.1.2 `uom` (default) | Removed; use the unit-suffixed accessors ([change 6](#changes-that-apply-to-every-crate)) |
| `nexrad-model` 0.1.0 `serde`, `chrono` | Unchanged |
| `nexrad-model` 0.1.0 `uom` | Removed; use the unit-suffixed accessors ([change 6](#changes-that-apply-to-every-crate)) |
| **`nexrad` 0.0.3 items** | |
| `decode::decode_file(&Vec<u8>) -> Result<model::DataFile>` | `nexrad::load(Vec<u8>) -> Result<Scan>` (decompresses as needed), or `nexrad_decode::messages::decode_messages(&[u8])` for decompressed message bytes |
| `decompress::decompress_file(&[u8]) -> Result<Vec<u8>>` | Not needed with `load`/`File::scan`; per record, `nexrad_data::volume::File::new(bytes).records()?` then `Record::decompress()` |
| `file::is_compressed(&[u8])` | `nexrad_data::volume::Record::compressed()` per LDM record (`File::compressed()` reports whole-file gzip compression, which 0.0.3 did not handle) |
| `download::list_files(&str, &NaiveDate) -> Result<Vec<FileMetadata>>` | `nexrad::list_scans(&str, NaiveDate) -> Result<Vec<Identifier>>` or `nexrad_data::aws::archive::list_files(&str, NaiveDate)` |
| `download::download_file(&FileMetadata) -> Result<Vec<u8>>` | `nexrad::download(&Identifier) -> Result<Scan>`, or `nexrad_data::aws::archive::download_file(&Identifier) -> Result<File>` |
| `file::FileMetadata` (`site()`, `date()`, `identifier()`) | `nexrad_data::aws::archive::Identifier` (`site()`, `date_time()`, `name()`) |
| `model::DataFile` (`volume_header()`, `elevation_scans()`) | `nexrad_model::data::Scan` (`site()`, `coverage_pattern()`, `sweeps()`); the volume header is `nexrad_data::volume::File::header()` |
| `model::Message31` (`header()`, `reflectivity_data()`, `velocity_data()`, `sw_data()`, `zdr_data()`, `phi_data()`, `rho_data()`, `cfp_data()`, `volume_data()`, `elevation_data()`, `radial_data()`) | Model level: `nexrad_model::data::Radial` (`reflectivity()`, `velocity()`, `spectrum_width()`, `differential_reflectivity()`, `differential_phase()`, `correlation_coefficient()`, `clutter_filter_power()`, `moment(product)`). Protocol level: `nexrad_decode::messages::digital_radar_data::Message` (`header()`, `*_data_block()`) |
| `model::Message31Header` (`azm()`, `elev()`, `elev_num()`, `radial_status()`, `ray_date()`, `ray_time()`, ...) | `Radial::azimuth_angle_degrees()`, `elevation_angle_degrees()`, `elevation_number()`, `radial_status()`, `collection_time()`; or `digital_radar_data::Header` accessors |
| `model::DataMoment` / `GenericData` (`moment_data()`, `scale()`, `offset()`, `number_data_moment_gates()`, ...) | `nexrad_model::data::MomentData` (`iter()`/`values()` decode gates to `MomentValue`; `scale()`, `offset()`, `gate_count()`, `raw_bytes()`) |
| `model::VolumeData`, `ElevationData`, `RadialData` | `nexrad_decode::messages::digital_radar_data::{VolumeDataBlock, ElevationDataBlock, RadialDataBlock}`; site position is also `Scan::site()` |
| `model::MessageHeader`, `VolumeHeaderRecord`, `DataBlockHeader` | `nexrad_decode::messages::MessageHeader`, `nexrad_data::volume::Header`, `digital_radar_data::DataBlockId` |
| `result::Error::{FileError, DeserializationError, DecompressionError}` | `nexrad::Error::Io`, `Decode`, `Data` |
| `result::Error::{S3GeneralError, S3ListObjectsError, S3GetObjectError, S3StreamingError}` | `nexrad::Error::Data(nexrad_data::Error::Aws(AwsError::..))` and `nexrad::Error::NoDataAvailable` |
| **`nexrad-model` 0.1.0** (new in 1.0: `Product`, `VolumeCoveragePattern`, `ElevationCut`, `VcpNumber`, `SweepField`, `CartesianField`, `VerticalField`, `GateStatus`, the `geo` module and the `meta::registry` site registry) | |
| `Radial::new(..)` | `Radial::builder(..)` (see the [nexrad-model table](#nexrad-model-100-rc2-to-10)) |
| `Radial::specific_differential_phase()` | Removed; `Radial::clutter_filter_power()` and `Radial::moment(Product)` are new |
| `MomentData::from_fixed_point(scale, offset, values)` | `MomentData::builder(product, values)` with `first_gate_range_meters`, `gate_interval_meters`, `data_word_size`, `scale`, `offset` and `build()` |
| `MomentData::values()` | Unchanged; `iter()` avoids the allocation. `MomentValue` gains `CfpStatus` |
| `RadialStatus::{VolumeScanStart, VolumeScanEnd, ElevationStartVCPFinal}` | `ScanStart`, `ScanEnd`, `ElevationStartVcpFinal`; new `Unknown(u8)` |
| `Scan::new(coverage_pattern_number: u16, sweeps)` | `Scan::new(VcpNumber, Option<VolumeCoveragePattern>, sweeps)` or `Scan::with_site(site, VcpNumber, Option<VolumeCoveragePattern>, sweeps)` |
| `Scan::coverage_pattern_number() -> u16` | `-> VcpNumber`; `.number()` gives the `u16` |
| `Scan::sweeps() -> &Vec<Sweep>`, `Sweep::radials() -> &Vec<Radial>` | Return slices |
| `Site::new(id, lat: f32, lon: f32, height, feedhorn_height)` | `Site::builder(id, lat: f64, lon: f64).site_elevation_meters(..).feedhorn_height_meters(..).build()` |
| `Site::latitude()`/`longitude() -> f32` | `-> f64` |
| `Site::height_meters()` / `height()` | `site_elevation_meters()`; `height()` is removed |
| `Site::feedhorn_height_meters()` / `feedhorn_height()` | `feedhorn_height_meters()` unchanged; `feedhorn_height()` is removed |
| `uom` accessors (`Radial::azimuth()`, `Site::feedhorn_height()`, ...) returning `uom::si::f32` | Removed; `azimuth_angle_degrees()`, `feedhorn_height_meters()`, ... ([change 6](#changes-that-apply-to-every-crate)) |
| `Error::ElevationMismatchError` | `Error::ElevationMismatch` |
| **`nexrad-decode` 0.1.2** (decoded from a reader; 1.0 decodes from a byte slice and borrows from it) | |
| `messages::decode_messages<R: Read + Seek>(&mut R) -> Result<Vec<Message>>` | `messages::decode_messages(&[u8]) -> Result<Vec<Message<'_>>>` |
| `messages::decode_message_header(&mut R)` | `messages::MessageHeader::parse(&[u8])` |
| `messages::decode_message_contents(&mut R, MessageType)`, `digital_radar_data::decode_digital_radar_data`, `rda_status_data::decode_rda_status_message`, `volume_coverage_pattern::decode_volume_coverage_pattern`, `clutter_filter_map::decode_clutter_filter_map` | `decode_messages`, which decodes every supported message type |
| `messages::message_header::MessageHeader` (module) | `messages::MessageHeader` |
| `MessageContents::Other` for most types | Typed variants for most message types (`DigitalRadarDataLegacy`, `ConsoleMessage`, `ClutterCensorZones`, `RequestForData`, ...); `Other` remains for the rest |
| Public struct fields on `digital_radar_data::{Message, Header, VolumeDataBlock, ..}` | Accessor methods: `m.header()`, `m.volume_data_block()`, `m.reflectivity_data_block()`, ...; the 0.1.2 names (`azimuth_angle()`, `latitude()`, `lrtup()`, `site_height()`, ...) follow the [nexrad-decode tables](#nexrad-decode-100-rc3-to-10): plain numbers gain a unit suffix and the `uom` quantities are removed |
| `digital_radar_data::Message::specific_diff_phase_data_block` | Removed |
| `digital_radar_data::ScaledMomentValue`, `GenericDataBlock::decoded_values()` | `DataBlock::moment_data()?` then `MomentData::iter()` (feature `model`) |
| `summarize` (`messages`, `MessageSummary`, `ScanSummary`) | Removed; see [nexrad-decode](#nexrad-decode-from-100-rc3) |
| `Error::FileError`, `DeserializationError` | Removed; decoding no longer does I/O or uses bincode |
| `Error::DecodingError(String)`, `MessageMissingDateError` | `Error::Decoding(String)`, `MessageMissingDate` |
| **`nexrad-data` 0.2.0** | |
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
| `download_chunk(site, &id) -> Result<(ChunkIdentifier, Chunk)>` | `download_chunk(&id) -> Result<DownloadedChunk>` |
| `result::Error::{FileError, CompressedDataError, UncompressedDataError, AWS}` | `Error::{Io, CompressedRecord, Aws}`; `Record::decompress` returns an uncompressed record unchanged |
| `result::Error::{DeserializationError, DecompressionError}` | Removed |
| `result::aws::AWSError::*Error` variants (`S3ListObjectsError`, `DateTimeError`, ...) | `AwsError::*` without the suffix (`S3ListObjects`, `DateTime`, ...); `PollingAsyncError` removed |

## Worked examples

Each flow is shown for the 0.x releases that had an equivalent, the last release candidate and 1.0. The 1.0 versions
compile against the 1.0 crates; the older ones come from those releases' documentation and examples.

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
            println!("{} gates", reflectivity.gate_count());
            for value in reflectivity.iter() {
                match value {
                    MomentValue::Value(dbz) => { let _ = dbz; }
                    MomentValue::BelowThreshold | MomentValue::RangeFolded => {}
                    MomentValue::CfpStatus(_) => {}
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
`Scan` (`decompress` handles gzip-wrapped files; `scan` decompresses LDM records).

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
        let scan = nexrad::download(id).await?;
        println!("{} sweeps", scan.sweeps().len());
    }
    // The newest archived volume, whatever the date: nexrad::download_latest("KDMX").await?

    let stream = nexrad::stream("KDMX", |d| tokio::time::sleep(d));
    let mut stream = std::pin::pin!(stream);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("{} ({} attempts)", chunk.identifier.name(), chunk.attempts);
    }
    // Or one Scan per complete volume: nexrad::stream_scans("KDMX", |d| tokio::time::sleep(d))
    Ok(())
}
```

With `nexrad-data` directly, the archive calls are `list_files("KDMX", date)` and
`download_file(id).await?.decompress()?.scan()?`; for real time, configure retries on a `PollConfig` and assemble
chunks into scans (on `wasm32`, pass a timer such as `|d| gloo_timers::future::sleep(d)`):

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

### Render a reflectivity sweep to PNG

0.x had no rendering library; the `render` example in `nexrad` 0.0.3 rasterised `Message31` gates by hand into a PPM
file.

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

    let scale = default_scale(Product::Reflectivity); // a ColorScale; reflectivity_scale().into() is the same
    let options = RenderOptions::new(1024, 1024);
    let result = render_sweep(&field, &scale, &options)?;
    result.save_png("reflectivity.png")?;

    if let Some(px) = result.metadata().km_to_pixel_distance(100.0) {
        println!("100 km is {px} pixels");
    }
    Ok(())
}
```

## Appendix: complete change tables

Every removed, renamed or re-typed public item between each release candidate and 1.0, with the pull request that
changed it. Added items that replace nothing are in the CHANGELOGs.

### nexrad-model 1.0.0-rc.2 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.2 | 1.0 | PR |
|---|---|---|
| `BinaryData<T>` (and its impls) | Not public; it appeared in no public signature | #162 |
| `data::VCPNumber` | `data::VcpNumber`, a `u16` newtype with associated constants named by pattern number (below) | #155, #160, #188, #205 |
| `data::CFPStatus` (variants unchanged) | `data::CfpStatus` | #160 |
| `data::CFPMomentData` (`from_fixed_point`, `iter`, `values`, `DataMoment` impl) | `data::MomentData` with `Product::ClutterFilterPower` | #170 |
| `data::CFPMomentValue::{Status, Value}` | `data::MomentValue::{CfpStatus, Value}` | #170 |
| `data::DataMoment` trait (`data_word_size`, `first_gate_range`, `first_gate_range_km`, `gate_count`, `gate_interval`, `gate_interval_km`, `offset`, `raw_gate_values`, `raw_values`, `scale`) | Inherent methods on `MomentData`, same names and signatures except `raw_values`, which is `raw_bytes`; remove the import | #170 |
| `data::ChannelConfiguration::SZ2Phase` | `Sz2Phase` | #160 |
| `data::WaveformType::{CS, CDW, CDWO, B, SPP, Unknown}` | `{ContiguousSurveillance, ContiguousDopplerWithAmbiguityResolution, ContiguousDopplerWithoutAmbiguityResolution, Batch, StaggeredPulsePair, Unknown(u8)}` | #160, #205 |
| `data::ChannelConfiguration::Unknown`, `data::PulseWidth::Unknown` | `Unknown(u8)` | #205 |
| `data::VCPNumber::number(&self)`, `from_number(u16)`, `description()` | `number(self)`, `VcpNumber::new(u16)` (or `From<u16>`); `description()` removed, and `Display` prints `VCP 212` without it | #205 |
| `data::RadialStatus::ElevationStartVCPFinal` | `ElevationStartVcpFinal` | #160 |
| `data::Product` | `#[non_exhaustive]` | #155 |
| `data::Product::moment_data(&Radial)`, `cfp_moment_data(&Radial)` | `Radial::moment(Product)` | #170 |
| `data::MomentData::from_fixed_point(u16, u16, u16, u8, f32, f32, Vec<u8>) -> Self` | `MomentData::builder(Product, Vec<u8>)` → `MomentDataBuilder` with `gate_count`, `first_gate_range_meters`, `gate_interval_meters`, `data_word_size`, `scale`, `offset`, `build() -> Result<MomentData>` | #170, #185, #210 |
| `data::MomentData::gate_count() -> u16` | `-> usize` | #185 |
| `data::Radial::new(i64, u16, f32, f32, RadialStatus, u8, f32, Option<MomentData> × 6, Option<CFPMomentData>)` | `Radial::builder(u16, f32, u8, f32)` → `RadialBuilder` with `collection_timestamp_millis`, `azimuth_spacing_degrees`, `radial_status`, `moment(data)` (replacing the per-product setters), `build() -> Result<Radial>` | #165, #170, #185 |
| `data::Sweep::new(u8, Vec<Radial>) -> Self` | `-> Result<Self>` | #185 |
| `data::Sweep::from_radials(Vec<Radial>) -> Vec<Sweep>` | `Sweep::split_by_elevation_number(Vec<Radial>) -> Vec<Sweep>` | #210 |
| `data::Radial::clutter_filter_power() -> Option<&CFPMomentData>` | `-> Option<&MomentData>` | #170 |
| `data::Radial::{azimuth, azimuth_spacing, elevation_angle}() -> uom::si::f32::Angle` | Removed; `azimuth_angle_degrees()`, `azimuth_spacing_degrees()`, `elevation_angle_degrees()` | #164, #202 |
| `data::Sweep::elevation_angle() -> Option<uom::si::f32::Angle>` | Removed; `elevation_angle_degrees()` | #164, #202 |
| `data::DataMoment::{first_gate_range, gate_interval}() -> uom::si::f64::Length` | Removed; `MomentData::{first_gate_range_km, gate_interval_km}()` | #170, #202 |
| `data::Scan::coverage_pattern_number() -> VCPNumber` | `-> VcpNumber` | #160 |
| `data::VCPNumber::{Precipitation12, ClearAirLongPulse31, ClearAirShortPulse32, ClearAir35, PrecipitationMpda112, PrecipitationSz2_212, GeneralSurveillance215}` | Constants `VcpNumber::{VCP_12, VCP_31, VCP_32, VCP_35, VCP_112, VCP_212, VCP_215}`, as in `nexrad_decode::messages::VcpNumber`, plus `VCP_11`, `VCP_21`, `VCP_121`, `VCP_211` and `VCP_221`; `Unknown(n)` is `VcpNumber::new(n)` | #188, #205 |
| `data::Radial` (no PRF context) | Adds `nyquist_velocity_mps()`, `unambiguous_range_km() -> Option<f32>`; `RadialBuilder::{nyquist_velocity_mps, unambiguous_range_km}` | #188 |
| `data::Scan` | Adds `elevation_cut(&Sweep) -> Option<&ElevationCut>`, `sweeps_with_waveform(WaveformType) -> impl Iterator<Item = &Sweep>` | #188 |
| `data::Scan::coverage_pattern() -> &VolumeCoveragePattern` | `-> Option<&VolumeCoveragePattern>` | #208 |
| `data::Scan::new(VolumeCoveragePattern, Vec<Sweep>)`, `with_site(Site, VolumeCoveragePattern, Vec<Sweep>)` | `new(VcpNumber, Option<VolumeCoveragePattern>, Vec<Sweep>)`, `with_site(Site, VcpNumber, Option<VolumeCoveragePattern>, Vec<Sweep>)` | #208 |
| `data::ElevationCut::new(f64, ChannelConfiguration, WaveformType, ..)` (22 arguments) | `ElevationCut::builder(f32, WaveformType, ChannelConfiguration)` → `ElevationCutBuilder`, `build() -> Result<ElevationCut>`; note the argument order | #165, #182, #210 |
| `data::ElevationCut::{reflectivity, velocity, spectrum_width, differential_reflectivity, differential_phase, correlation_coefficient}_threshold_db()` | `{reflectivity, velocity, spectrum_width, differential_reflectivity, differential_phase, correlation_coefficient}_snr_threshold_db()`; the builder setters are renamed the same way | #210 |
| `data::ElevationCut::elevation_angle_degrees() -> f64` | `-> f32` | #182 |
| `data::VolumeCoveragePattern::new(u16, ..)` (14 arguments) | `VolumeCoveragePattern::builder(VcpNumber)` → `VolumeCoveragePatternBuilder`, `build() -> Result<VolumeCoveragePattern>`; convert a number with `VcpNumber::new(n)` | #165, #205, #210 |
| `data::VolumeCoveragePattern::number_of_elevation_cuts() -> usize` | `elevation_cut_count() -> usize` | #210 |
| `data::VolumeCoveragePattern::pattern_number() -> VCPNumber` | `-> VcpNumber` | #160 |
| `data::SweepField::new_empty(label, unit, f32, Vec<f32>, f32, f64, f64, usize)` | `SweepField::builder(label, unit)` → `SweepFieldBuilder`, `build() -> Result<SweepField>`, which validates the geometry | #165, #185 |
| `data::SweepField::value_at_polar(f32, f64)` | `value_at_polar(f64, f64)` | #164 |
| `data::SweepField::elevation_degrees()` | `elevation_angle_degrees()` | #182 |
| `data::SweepField::azimuths() -> &[f32]` | `azimuth_angles_degrees() -> &[f32]` | #182 |
| `data::SweepField::radial_status_mut(usize) -> &mut [GateStatus]` | `radial_statuses_mut(usize) -> Option<&mut [GateStatus]>` | #182, #185 |
| `data::SweepField::radial_values(usize) -> &[f32]`, `radial_values_mut(usize) -> &mut [f32]`, `radial_statuses(usize) -> &[GateStatus]` | `-> Option<&[f32]>`, `-> Option<&mut [f32]>`, `-> Option<&[GateStatus]>` | #185 |
| `data::SweepField::get(usize, usize) -> (f32, GateStatus)` | `-> Option<(f32, GateStatus)>` | #185 |
| `data::SweepField::set(usize, usize, f32, GateStatus)` | `-> Result<()>` | #185 |
| `data::CartesianField::new(label, unit, GeoExtent, usize, usize) -> Self` | `-> Result<Self>` | #185 |
| `data::CartesianField::get(usize, usize) -> (f32, GateStatus)`, `set(..)` | `get -> Option<(f32, GateStatus)>`, `set -> Result<()>` | #185 |
| `data::VerticalField::new(label, unit, (f64, f64), (f64, f64), usize, usize) -> Self` | `-> Result<Self>` | #185 |
| `data::VerticalField::get(usize, usize) -> (f32, GateStatus)`, `set(..)` | `get -> Option<(f32, GateStatus)>`, `set -> Result<()>` | #185 |
| `data::GateStatus::{Valid, BelowThreshold, RangeFolded, NoData}` | Adds `Filtered` | #187 |
| `impl From<&data::MomentValue> for data::GateStatus` | `impl From<data::MomentValue> for data::GateStatus` | #187 |
| `data::GateStatus` | `#[non_exhaustive]`; add a wildcard arm | #211 |
| `data::Radial::collection_timestamp() -> i64` | `collection_timestamp_millis() -> i64`, milliseconds since the Unix epoch, UTC | #182 |
| `data::VolumeCoveragePattern::doppler_velocity_resolution() -> f32` | `doppler_velocity_resolution_mps() -> f32` | #182 |
| `data::VolumeCoveragePattern::{sails_cuts, mrle_cuts}() -> u8` | `{sails_cut_count, mrle_cut_count}() -> u8` | #182 |
| `geo::PolarPoint::{azimuth_degrees, elevation_degrees}: f32` | `{azimuth_angle_degrees, elevation_angle_degrees}: f64` | #164, #182 |
| `geo::RadarCoordinateSystem::gate_location(f32, f32, usize, f64, f64)` | Removed; call `polar_to_geo` with `range_km = first_gate_km + gate_index * gate_interval_km` | #164, #PR |
| `geo::RadarCoordinateSystem::geo_to_polar(GeoPoint, f32) -> PolarPoint` (flat-earth range) | `geo_to_polar(GeoPoint, f64) -> Option<PolarPoint>`, the exact inverse of `polar_to_geo`; ranges change, and `None` replaces a non-finite or negative range | #164, #188, #210 |
| `geo::RadarCoordinateSystem` | Adds `beam_height_meters(f64, f64) -> f64`, `ground_range_km(f64, f64) -> f64` | #188 |
| `meta::Site::new([u8; 4], f32, f32, i16, u16)` | `Site::builder([u8; 4], f64, f64)` → `SiteBuilder` with `site_elevation_meters`, `feedhorn_height_meters`, `build() -> Result<Site>` | #164, #165, #182, #185 |
| `meta::Site::identifier() -> &[u8; 4]` | `-> &str` | #185 |
| `meta::Site::identifier_string() -> String` | Removed; `identifier()` | #185 |
| `meta::Site::{latitude, longitude}() -> f32` | `-> f64` | #164 |
| `meta::Site::height_meters() -> i16` | `site_elevation_meters() -> i16`, above mean sea level | #182 |
| `meta::Site::tower_height_meters() -> u16` | `feedhorn_height_meters() -> u16`, above ground level | #182 |
| `meta::Site::{height, tower_height}() -> uom::si::f32::Length` | Removed; `site_elevation_meters()`, `feedhorn_height_meters()` | #164, #182, #202 |
| Feature `uom` | Removed | #202 |
| `meta::registry::SiteEntry` | `#[non_exhaustive]` | #182 |
| `meta::registry::SiteEntry::{latitude, longitude}: f32` | `f64` | #164 |
| `meta::registry::SiteEntry::elevation_meters: i16` | `site_elevation_meters: i16` | #182 |
| `meta::registry::SiteEntry::state: &'static str` | `Option<&'static str>`, `None` for a site outside the United States | #211 |
| `meta::registry::nearest_site(f32, f32)` | `nearest_site(f64, f64)` | #164 |
| `meta::registry::SiteEntry::to_site() -> Site` | `-> Result<Site>`; every registry entry is valid | #185 |
| `result::Error` | `#[non_exhaustive]`, `Clone`, `PartialEq`; new `InvalidGeometry { parameter, reason }`, `InvalidMomentData { parameter, reason }`, `MomentDataLength { expected, actual }`, `InvalidCoveragePattern { parameter, reason }`, `InvalidSite { parameter, reason }`, `IndexOutOfBounds { row, column, rows, columns }` | #155, #185, #210 |
| `result::Error::ElevationMismatch` (unit variant) | `ElevationMismatch { expected: u8, actual: u8 }` | #185 |
| serde: `data::MomentData` keys `first_gate_range`, `gate_interval`, `values` (a sequence of integers) | `first_gate_range_meters`, `gate_interval_meters`, `raw_bytes` (a byte string) | #189 |
| serde: `data::{SweepField, CartesianField, VerticalField}` key `status` | `statuses` | #189 |
| serde: `data::ElevationCut` keys `*_threshold_db` | `*_snr_threshold_db` | #210 |
| serde: `meta::Site` key `identifier` as `[u8; 4]` (an array of four numbers) | A four-character string (`"KTLX"`) | #189 |
| serde: `data::CfpStatus` variants in snake_case | PascalCase variant names | #189 |
| serde: `Deserialize` for `data::{SweepField, CartesianField, VerticalField, MomentData, Radial, Sweep}` and `meta::Site` (no validation) | Validates as the constructors and builders do | #189 |
| serde: `Deserialize` for `data::{ElevationCut, VolumeCoveragePattern}` (no validation) | Validates as `ElevationCutBuilder::build` and `VolumeCoveragePatternBuilder::build` do | #210 |

The facade's `extract_field` and friends moved to model methods; see the [nexrad table](#nexrad-100-rc4-to-10).

</details>

### nexrad-decode 1.0.0-rc.3 to 1.0

<details>
<summary>Show the table</summary>

All paths are under `nexrad_decode::messages` unless they start with `result` or `summarize`.

**Types and enum variants**

| 1.0.0-rc.3 | 1.0 | PR |
|---|---|---|
| `MessageType::RDAStatusData`, `RDAConsoleMessage`, `RDAVolumeCoveragePattern`, `RDAControlCommands`, `RDAAdaptationData`, `RDAPRFData`, `RDALogData` | `Rda...` (`RdaPrfData` for `RDAPRFData`) | #160 |
| `MessageType::RDADigitalRadarDataGenericFormat`, `RDADigitalRadarData`, `RDAPerformanceMaintenanceData`, `RDAClutterFilterBypassMap`, `RDAClutterFilterMap` | `DigitalRadarData`, `DigitalRadarDataLegacy`, `PerformanceMaintenanceData`, `ClutterFilterBypassMap`, `ClutterFilterMap`, matching `MessageContents` | #191 |
| `MessageType::RPGVolumeCoveragePattern`, `RPGConsoleMessage` | `RpgVolumeCoveragePattern`, `RpgConsoleMessage` | #160 |
| `MessageType::RPGClutterCensorZones`, `RPGRequestForData` | `ClutterCensorZones`, `RequestForData` | #191 |
| `MessageType::RDALoopBackTest`, `RPGLoopBackTest` | `RdaLoopbackTest`, `RpgLoopbackTest` | #191 |
| `MessageContents::VolumeCoveragePattern` | `RdaVolumeCoveragePattern` (`MessageType` keeps `RdaVolumeCoveragePattern`, type 5, apart from `RpgVolumeCoveragePattern`, type 7) | #191 |
| `MessageType::Spare1` (14), `ReservedFAARMSOnly1..5` (16, 17, 24, 25, 26), `Reserved1..5` (20–23, 29) | `MessageType::Reserved(u8)` holding the type number; 29 is `MessageType::ModelData` | #166, #206 |
| `MessageType` with `#[repr(u8)]` discriminants | No discriminants; `number()` replaces `as u8`, `from_number(u8)`; `#[non_exhaustive]` | #155, #166 |
| `MessageContents::RDAStatusData`, `RDAControlCommands`, `RDAPRFData`, `RDALogData`, `RDAAdaptationData` | `RdaStatusData`, `RdaControlCommands`, `RdaPrfData`, `RdaLogData`, `RdaAdaptationData`; `#[non_exhaustive]` | #155, #160 |
| `RedundantChannel::ORDASingleChannel`, `ORDARedundantChannel1`, `ORDARedundantChannel2` | `OrdaSingleChannel`, `OrdaRedundantChannel1`, `OrdaRedundantChannel2`; `#[non_exhaustive]` | #155, #160 |
| `MessageHeader` (`#[repr(C)]`, public fields, zerocopy traits) | `MessageHeader<'a>` with accessors; `parse(bytes)?` replaces `ref_from_prefix`, `SIZE` replaces `size_of::<MessageHeader>()`, `into_owned()` gives `MessageHeader<'static>` | #166 |
| `MessageHeaders::Single(&'a MessageHeader)`, `Multiple(Vec<&'a MessageHeader>)` | `Single(MessageHeader<'a>)`, `Multiple(Vec<MessageHeader<'a>>)` | #166 |
| `digital_radar_data::VolumeCoveragePattern::{VCP12, VCP31, VCP35, VCP112, VCP212, VCP215, Unknown}` | `messages::VcpNumber`, a `u16` newtype: constants `VcpNumber::{VCP_11, VCP_12, VCP_21, VCP_31, VCP_32, VCP_35, VCP_112, VCP_121, VCP_211, VCP_212, VCP_215, VCP_221}`, and `VcpNumber::new(n)` for `Unknown(n)` | #160, #163, #188, #205 |
| `digital_radar_data::DataMomentPointer`, `DataMomentPointerType`, `DataMomentGenericPointerType` | Removed | #166 |
| `digital_radar_data::CompressionIndicator::CompressedBZIP2`, `CompressedZLIB`, `FutureUse` | `CompressedBzip2`, `CompressedZlib`, `Unknown(u8)` | #160, #191 |
| `digital_radar_data::ProcessingStatus::RxRNoise`, `CBT`, `Other(u16)` | `RxrNoise`, `Cbt`, `Unknown(u16)` | #160, #191 |
| `digital_radar_data::SpotBlankingStatus` | `SpotBlankingFlags` (`rda_status_data::SpotBlankingStatus` keeps its name) | #191 |
| `digital_radar_data::RadialStatus::ElevationStartVCPFinal` | `ElevationStartVcpFinal` | #160 |
| `digital_radar_data::RadialStatus::VolumeScanStart`, `VolumeScanEnd` | `ScanStart`, `ScanEnd`, as in `nexrad_model::data::RadialStatus` | #205 |
| `digital_radar_data::DataBlockName` | `#[non_exhaustive]` | #155 |
| The code enums with an `Unknown(n)` or `Reserved(n)` variant in `clutter_censor_zones`, `clutter_filter_map`, `digital_radar_data` (except `RadialStatus`), `rda_status_data` and `volume_coverage_pattern` (except `WaveformType`, `ChannelConfiguration`, `PulseWidth`) | `#[non_exhaustive]`; add a wildcard arm | #207 |
| `rda_status_data::RDAStatus`, `RDABuildNumber`, `RMSControlStatus` | `RdaStatus`, `RdaBuildNumber`, `RmsControlStatus` | #160 |
| `rda_status_data::RMSControlStatus::{NonRMS, RMSInControl, RDAInControl}` | `RmsControlStatus::{NonRms, RmsInControl, RdaInControl}` | #160 |
| `rda_status_data::CommandAcknowledgement::RemoteVCPReceived` | `RemoteVcpReceived` | #160 |
| `rda_status_data::TransitionPowerSourceStatus::OK`, `Unknown`, `Other(u16)` | `Ok`, `Unknown(u16)` (code 4 is `Unknown(4)`) | #160, #191 |
| `rda_status_data::SpotBlankingStatus::Other(u8)` | `Unknown(u16)`, no longer truncating the wire value | #191 |
| `rda_status_data::RDABuildNumber::{Build12_0, .., Build24_0, Unknown(f32)}`, `as_float()`, `is_known()` | `RdaBuildNumber` newtype: `from_raw(u16)`, `raw()`, `major()`, `minor()`, `Display`, `Ord` | #191 |
| `rda_status_data::alarm::Message`, `alarm::get_alarm_message(u16)` | `alarm::Definition`, `alarm::definition(u16)` | #191 |
| `rda_status_data::VolumeCoveragePatternNumber` (`number() -> i16`, `local()`, `remote()`) | `VcpSelection` (`number() -> VcpNumber`, `source() -> VcpSource`) | #163, #191 |
| `volume_coverage_pattern::ChannelConfiguration::SZ2Phase`, `UnknownPhase` | `Sz2Phase`, `Unknown(u8)` | #160, #191 |
| `volume_coverage_pattern::PulseWidth::Unknown`, `PatternType::Unknown` | `PulseWidth::Unknown(u8)`, `PatternType::Unknown(u16)` | #191 |
| `volume_coverage_pattern::WaveformType::{CS, CDW, CDWO, B, SPP, Unknown}` | `{ContiguousSurveillance, ContiguousDopplerWithAmbiguityResolution, ContiguousDopplerWithoutAmbiguityResolution, Batch, StaggeredPulsePair, Unknown(u8)}` | #160, #191 |
| `clutter_censor_zones::Message::regions() -> &[raw Region]` (public `Integer2`/`Code2` fields) | `-> &[clutter_censor_zones::Region<'a>]` with accessors and `OperatorSelectCode` | #166 |
| `summarize` module (`messages`, `MessageSummary`, `MessageGroupSummary`, `VCPInfo`, `VCPElevationInfo`, `RDAStatusInfo`) | Removed; iterate `decode_messages(..)` and match on `message.contents()` | #168 |
| `result::Error::Io(std::io::Error)`, `From<std::io::Error>` | Removed | #168 |
| `result::Error` | `#[non_exhaustive]`, `Clone`, `PartialEq`; new `MissingCoveragePatternNumber` (feature `model`), `NegativeFirstGateRange` (feature `model`), `SegmentOutOfSequence` and (feature `model`) `Model(nexrad_model::Error)` | #155, #169, #185, #190, #208 |
| `result::Error::Decoding(String)`, `InvalidMessageLength { message_type, delta }`, `DataSpansSegmentBoundary { position }` | Removed (never constructed) | #190 |
| `result::Error::UnexpectedEof`, `MessageMissingDate` | `UnexpectedEof { offset }`, `MessageMissingDate { message_type }` | #190 |
| `result::Error::UnknownDataBlockType { block_type: String }` | `UnknownDataBlockType { block_type: [u8; 3], offset }`, no longer returned by decoding for an unknown `'R'` or `'D'` block | #190, #207 |
| `MessageContents::Other` | `MessageContents::Other { message_type: MessageType, data: Cow<'a, [u8]> }`; match `Other { .. }` | #190 |
| `digital_radar_data::DataBlock<'a, T>: Deref<Target = T>` | No `Deref`; `inner()`, `into_inner()` (`block.inner().vcp_number()`) | #190 |
| `decode_messages(&[u8]) -> Result<Vec<Message>>`, skipping undecodable messages | Same signature, returns the first error; also at `nexrad_decode::decode_messages` | #190 |
| (new) | `iter_messages(&[u8]) -> MessageIter`, `MessageIter::offset()`; root re-exports of `Message`, `MessageContents`, `MessageIter`, `MessageType` | #190 |
| (new) | `Message::into_owned()`, `MessageContents::into_owned()`, `MessageContents::message_type()`, `MessageHeaders::into_owned()` | #190 |

**Accessors**

| Type | 1.0.0-rc.3 | 1.0 | PR |
|---|---|---|---|
| `MessageHeader` | fields `segment_size`, `redundant_channel`, `message_type`, `sequence_number`, `date` + `time`, `segment_count`, `segment_number` | `segment_size_raw()`, `redundant_channel_raw()` / `redundant_channel()`, `message_type_raw()` / `message_type()`, `sequence_number()`, `date_time()`, `segment_count()`, `segment_number()` | #166, #173 |
| `MessageHeader` | `rda_redundant_channel()` | `redundant_channel()` | #173 |
| `MessageHeader` | (none) | `segment_size_bytes()` | #173 |
| `digital_radar_data::Header` | `azimuth_angle_raw()`, `elevation_angle_raw()`, `radial_length_raw()` | `azimuth_angle_degrees()`, `elevation_angle_degrees()`, `radial_length_bytes()` | #173 |
| `digital_radar_data::Header` | `azimuth_resolution_spacing_type() -> AzimuthResolutionSpacing` | `azimuth_resolution_spacing() -> AzimuthResolutionSpacing` | #173 |
| `digital_radar_data::Header` | `azimuth_resolution_spacing() -> Angle` (uom) | `azimuth_resolution_spacing_degrees() -> f32` | #173 |
| `digital_radar_data::Header` | `azimuth_indexing_mode_degrees() -> Option<f64>` | `-> Option<f32>` | #191 |
| `digital_radar_data::VolumeDataBlock` | `lrtup_raw()`, `latitude_raw()`, `longitude_raw()`, `site_height_raw()`, `tower_height_raw()` | `lrtup_bytes()`, `latitude_degrees()`, `longitude_degrees()`, `site_height_meters()`, `tower_height_meters()` | #173 |
| `digital_radar_data::VolumeDataBlock` | `calibration_constant()`, `system_differential_reflectivity()` | `calibration_constant_db()`, `system_differential_reflectivity_db()` | #173 |
| `digital_radar_data::VolumeDataBlock` | `horizontal_shv_tx_power_raw()`, `vertical_shv_tx_power_raw()` | `horizontal_shv_tx_power_kw()`, `vertical_shv_tx_power_kw()` | #173 |
| `digital_radar_data::VolumeDataBlock` | `initial_system_differential_phase_raw()` | `initial_system_differential_phase_degrees()` | #173 |
| `digital_radar_data::VolumeDataBlock` | `volume_coverage_pattern() -> VolumeCoveragePattern`, `volume_coverage_pattern_number()` | `vcp_number() -> VcpNumber`, `vcp_number_raw()` | #163, #173, #191 |
| `digital_radar_data::VolumeDataBlock` | `zdr_bias_estimate_weighted_mean()` | `zdr_bias_estimate_weighted_mean_raw()`, `zdr_bias_estimate_weighted_mean_db() -> Option<f32>` | #191 |
| `digital_radar_data::RadialDataBlock` | `lrtup_raw()`, `horizontal_channel_noise_level()`, `vertical_channel_noise_level()` | `lrtup_bytes()`, `horizontal_channel_noise_level_dbm()`, `vertical_channel_noise_level_dbm()` | #173 |
| `digital_radar_data::RadialDataBlock` | `horizontal_channel_calibration_constant()`, `vertical_channel_calibration_constant()` | `horizontal_channel_calibration_constant_db()`, `vertical_channel_calibration_constant_db()` | #173 |
| `digital_radar_data::RadialDataBlock` | (none) | `unambiguous_range_km()`, `nyquist_velocity_mps()` | #173 |
| `digital_radar_data::ElevationDataBlock` | `lrtup_raw()`, `calibration_constant()`, `atmos_raw()`, `atmos()` | `lrtup_bytes()`, `calibration_constant_db()`, `atmospheric_attenuation_raw()`, `atmospheric_attenuation_db_per_km()` | #173, #191 |
| `digital_radar_data::GenericDataBlockHeader` | (none) | `tover_db()`, `snr_threshold_db()`, `moment_size_bytes()` | #173 |
| `digital_radar_data::GenericDataBlock` | `moment_data()`, `into_moment_data()`, `cfp_moment_data()`, `into_cfp_moment_data()` | `DataBlock<'a, GenericDataBlock<'a>>::moment_data() -> Result<MomentData>`, `into_moment_data()` (`m.reflectivity_data_block().map(\|b\| b.moment_data())`) | #170 |
| `digital_radar_data::Message` | `site() -> Option<Site>` | `site() -> Result<Option<Site>>` | #185 |
| `digital_radar_data_legacy::Message` | `azimuth_angle()`, `elevation_angle()`, `calibration_constant()` | `azimuth_angle_degrees()`, `elevation_angle_degrees()`, `calibration_constant_db()` | #173 |
| `digital_radar_data_legacy::Message` | `surveillance_first_gate_range()`, `doppler_first_gate_range()`, `surveillance_gate_interval()`, `doppler_gate_interval()` | `..._meters()` | #173 |
| `digital_radar_data_legacy::Message` | `doppler_velocity_resolution()` | `doppler_velocity_resolution_mps()` | #173 |
| `digital_radar_data_legacy::Message` | `elevation_number() -> u16`, `radial_status() -> u16`, `vcp_number() -> u16` | `elevation_number() -> u8`, `radial_status() -> RadialStatus` + `radial_status_raw()`, `vcp_number() -> VcpNumber` + `vcp_number_raw()` | #173 |
| `volume_coverage_pattern::Header` | `message_size()`, `doppler_velocity_resolution()` | `message_size_halfwords()`, `doppler_velocity_resolution_mps()` | #173 |
| `volume_coverage_pattern::ElevationDataBlock` | `elevation_angle()`, `azimuth_rate()`, `sector_1_edge_angle()`, `sector_2_edge_angle()`, `sector_3_edge_angle()`, `ebc_angle()` | `elevation_angle_degrees()`, `azimuth_rate_degrees_per_second()`, `sector_N_edge_angle_degrees()`, `ebc_angle_degrees()` | #173 |
| `volume_coverage_pattern::ElevationDataBlock` | `reflectivity_threshold()`, `velocity_threshold()`, `spectrum_width_threshold()`, `differential_reflectivity_threshold()`, `differential_phase_threshold()`, `correlation_coefficient_threshold()` | `..._threshold_db()` | #173 |
| `clutter_filter_map::RangeZone` | `raw_op_code()`, `end_range() -> u16`, `end_range_uom()` | `op_code_raw()`, `end_range_km() -> u16`; `end_range_uom()` removed | #173, #202 |
| `request_for_data::Message` | `raw_data_request_type()` | `data_request_type_raw()` | #173 |
| `MessageHeader`, `digital_radar_data::{Header, VolumeDataBlock, ElevationDataBlock, RadialDataBlock, GenericDataBlockHeader}` | `uom` accessors (feature `uom`): `message_size()`, `segment_size()`, `azimuth_angle()`, `elevation_angle()`, `azimuth_indexing_mode()`, `radial_length()`, `lrtup()`, `latitude()`, `longitude()`, `site_height()`, `tower_height()`, `horizontal_shv_tx_power()`, `vertical_shv_tx_power()`, `initial_system_differential_phase()`, `unambiguous_range()`, `nyquist_velocity()`, `data_moment_range()`, `data_moment_range_sample_interval()`, `moment_size()`; feature `uom` | Removed; the unit-suffixed siblings listed in [change 6](#changes-that-apply-to-every-crate) | #202 |
| `rda_status_data::Message` | `raw_rda_status()`, `raw_operability_status()`, `raw_control_status()`, `raw_auxiliary_power_generator_state()`, `raw_horizontal_reflectivity_calibration_correction()`, `raw_data_transmission_enabled()`, `raw_rda_control_authorization()`, `raw_operational_mode()`, `raw_super_resolution_status()`, `raw_clutter_mitigation_decision_status()`, `raw_rda_scan_and_data_flags()`, `raw_rda_alarm_summary()`, `raw_command_acknowledgement()`, `raw_channel_control_status()`, `raw_spot_blanking_status()`, `raw_vertical_reflectivity_calibration_correction()`, `raw_transition_power_source_status()`, `raw_rms_control_status()`, `raw_performance_check_status()`, `raw_alarm_codes()`, `raw_signal_processor_options()` | The same stems with a `_raw` suffix (`rda_status_raw()`, ..., `signal_processor_options_raw()`) | #173 |
| `rda_status_data::Message` | `raw_rda_build_number()` | `build_number_raw()` | #173 |
| `rda_status_data::Message` | `raw_volume_coverage_pattern()`, `volume_coverage_pattern()` | `vcp_number_raw()`, `vcp_number()` (`Option<VcpSelection>` instead of `Option<VolumeCoveragePatternNumber>`) | #163, #191 |
| `rda_status_data::Message` | `bypass_map_generation_date()`, `bypass_map_generation_time()`, `clutter_filter_map_generation_date()`, `clutter_filter_map_generation_time()` | `..._raw()`; the `*_date_time()` accessors are unchanged | #191 |
| `rda_status_data::Message` | `downloaded_pattern_number() -> u16` | `downloaded_pattern_number_raw()`, `downloaded_pattern_number() -> VcpNumber` | #191 |
| `rda_status_data::Message` | `alarm_messages()` | `alarm_definitions()` | #191 |
| `digital_radar_data::Header` | `date()`, `time()`, `radar_identifier() -> String` | `date_raw()`, `time_raw()` (`date_time()` is the decoded value), `radar_identifier() -> Option<&str>` (`None` when not valid UTF-8) | #191 |
| `digital_radar_data::DataBlockId` | `data_name_raw()`, `data_block_name_str()`, `data_block_type_char()` | `data_block_name_raw()`; the other two removed (use `data_block_name()`/`data_block_name_raw()` and `data_block_type()`/`data_block_type_raw()`) | #191 |
| `digital_radar_data::RadialDataBlock` | `radial_flags()` | `radial_flags_raw()` | #191 |
| `digital_radar_data::GenericDataBlockHeader` | `number_of_data_moment_gates()` | `data_moment_gate_count()` | #191 |
| `digital_radar_data_legacy::Message` | `num_surveillance_gates()`, `num_doppler_gates()` | `surveillance_gate_count()`, `doppler_gate_count()` | #191 |
| `digital_radar_data_legacy::Message` | (none) | `nyquist_velocity_raw()`, `nyquist_velocity_mps()`, `atmospheric_attenuation_raw()`, `atmospheric_attenuation_db_per_km()` | #191 |
| `volume_coverage_pattern::Header` | `pattern_number()`, `number_of_elevation_cuts()`, `number_of_sails_cuts()`, `number_of_mrle_cuts()`, `number_of_base_tilts()` | `vcp_number_raw()` + `vcp_number()`, `elevation_cut_count()`, `sails_cut_count()`, `mrle_cut_count()`, `base_tilt_count()` | #191 |
| `volume_coverage_pattern::Header` | `vcp_sequencing()`, `vcp_supplemental_data()`, `vcp_sequencing_number_of_elevations()` | `vcp_sequencing_raw()`, `vcp_supplemental_data_raw()`, `vcp_sequencing_elevation_count()` | #191 |
| `volume_coverage_pattern::ElevationDataBlock` | `supplemental_data()`, `super_resolution_control()`; angles as `f64` | `supplemental_data_raw()`, `super_resolution_control_raw()`; angles as `f32` | #191 |
| `clutter_filter_map::Message` | `map_generation_date()`, `map_generation_time()`, `date_time()` | `map_generation_date_raw()`, `map_generation_time_raw()`, `map_generation_date_time()` | #191 |
| `clutter_filter_map::AzimuthSegment` | `azimuth_segment()` | `azimuth_segment_number()` | #191 |
| `clutter_filter_bypass_map::Message` | `bypass_map_generation_date()`, `bypass_map_generation_time()`, `date_time()`, `number_of_elevation_segments()` | `bypass_map_generation_date_raw()`, `bypass_map_generation_time_raw()`, `bypass_map_generation_date_time()`, `elevation_segment_count()` | #191 |
| `clutter_filter_bypass_map::ElevationSegment` | `segment_number()` | `elevation_segment_number()` | #191 |
| `rda_prf_data::Message` | `number_of_waveforms()` | `waveform_count()` | #191 |
| `rda_prf_data::WaveformPrfData` | `waveform_type() -> u16` | `waveform_type_raw()`, `waveform_type() -> WaveformType` | #191 |
| `rda_log_data::Message` | `identifier() -> &str`, `identifier_bytes()`, `compression_type()` | `identifier() -> Option<&str>` (was `""` when invalid), `identifier_raw()`, `compression_type_raw()` | #191 |
| `console_message::Message` | `text_bytes()` | `text_raw()` | #191 |
| `MessageHeader` | `segmented()` | `is_fixed_frame()` | #191 |
| `Message`, `MessageHeaders` | `is_segmented()`; `Message::offset()`, `size()` | `is_multi_segment()`; `offset_bytes()`, `size_bytes()` | #191 |
| `rda_status_data::Message` | `average_transmitter_power()`, `horizontal_reflectivity_calibration_correction()` | `average_transmitter_power_watts()`, `horizontal_reflectivity_calibration_correction_db()` | #173 |
| `rda_status_data::Message` | (none) | `vertical_reflectivity_calibration_correction_db()` | #173 |
| `console_message::Message` | `message_size()` | `message_size_bytes()` | #173 |
| `loopback_test::Message` | `message_size()` | `message_size_halfwords()` | #173 |
| `rda_log_data::Message` | `compressed_size()`, `decompressed_size()` | `compressed_size_bytes()`, `decompressed_size_bytes()` | #173 |
| `rda_adaptation_data::Message` | `raw_data()`, `read_integer4_at()`, `read_real4_at()`, `read_string_at()`, `total_size()` | Removed; use the typed accessors | #190 |
| `digital_radar_data::VolumeDataBlock` | `is_legacy()`, `spare()` | Removed; `lrtup_bytes()` is 44 for a legacy volume block | #190 |
| `digital_radar_data::RadialDataBlock` | `is_legacy()` | Removed; `lrtup_bytes()` is 20 for a legacy radial block | #190 |
| `digital_radar_data::GenericDataBlockHeader` | `reserved()` | Removed | #190 |
| `clutter_censor_zones::Region` | fields `start_range`, `stop_range`, `start_azimuth`, `stop_azimuth`, `elevation_segment_number`, `operator_select_code` | `start_range_km()`, `stop_range_km()`, `start_azimuth_degrees()`, `stop_azimuth_degrees()`, `elevation_segment_number()`, `operator_select_code()` / `operator_select_code_raw()` | #166 |

The `rda_adaptation_data` (type 18) and `performance_maintenance_data` (type 3) accessors keep their ICD mnemonic names.

</details>

### nexrad-data 1.0.0-rc.7 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.7 | 1.0 | PR |
|---|---|---|
| `result::aws` module, `result::aws::AWSError` | `result::AwsError` (also `nexrad_data::AwsError`), `#[non_exhaustive]` | #155, #160, #171 |
| `AWSError::S3GetObjectRequest(reqwest::Error)`, `S3ListObjects(reqwest::Error)`, `S3Streaming(reqwest::Error)` | Same names holding `Box<dyn Error + Send + Sync>`, which `source()` returns | #171 |
| `AWSError::PollingAsync` | Removed (never constructed) | #171 |
| `result::Error::AWS` | `Error::Aws` | #160 |
| `result::Error::Decompression(bzip2::Error)`, `From<bzip2::Error>` | Removed (never constructed); bzip2 failures are `Error::Io` | #171 |
| `result::Error` | `#[non_exhaustive]` | #155 |
| `volume::split_compressed_records(&[u8])` | Removed; `File::new(bytes).records()?` | #171 |
| `volume::Header::{extension_number, icao_of_radar, tape_filename}() -> Option<String>` | `-> Option<&str>` | #171 |
| `volume::File::header() -> Option<&Header>`; `#[repr(C)] volume::Header` with `zerocopy::{FromBytes, FromZeros, Immutable, KnownLayout, TryFromBytes}` | `header() -> Option<Header>`; `Header` without zerocopy traits, `Header::SIZE` replacing `size_of::<Header>()` | #181 |
| `volume::Record::decompress<'b>(&self) -> Result<Record<'b>>` | `decompress(&self) -> Result<Record<'static>>` | #171 |
| `aws::archive::list_files(&str, &NaiveDate)` | `list_files(&str, NaiveDate)` | #171 |
| `aws::archive::download_file(Identifier)` | `download_file(&Identifier)` | #171 |
| `aws::archive::Identifier::new(String) -> Identifier` | `Identifier::parse(&str) -> Result<Identifier>`, `impl FromStr` | #192 |
| `aws::archive::Identifier::{site() -> Option<&str>, date_time() -> Option<DateTime<Utc>>}` | `site() -> &str`, `date_time() -> DateTime<Utc>` | #192 |
| (new) | `aws::archive::Identifier::{version, is_model_data, is_gzip}()`, `Display`, `Debug`; `aws::archive::list_files_with_sizes`, `ListedFile` | #186, #192, #206 |
| `aws::archive::list_files` returning every object | Volume files only | #192 |
| `volume::File::scan()`, `coverage_pattern_message()` returning `Error::CompressedFile` for a gzip-wrapped file | Decompress the file themselves; `records()` still returns `CompressedFile` | #208 |
| `volume::File::scan()`, `aws::realtime::assemble_volume` returning `Error::MissingCoveragePattern` without a type 5 message | Return a scan with `coverage_pattern()` `None`; `Error::Decode(MissingCoveragePatternNumber)` when no coverage pattern number is found | #208 |
| `aws::realtime::VolumeIndex::new(usize) -> VolumeIndex` | `new(usize) -> Result<VolumeIndex>` | #192 |
| `aws::realtime::list_chunks_in_volume(&str, VolumeIndex, usize)` | `list_chunks_in_volume(&str, VolumeIndex)` | #149, #192 |
| `AwsError::S3GetObject(Option<String>)` | `S3GetObject { status: u16, body: Option<String> }` | #192 |
| `AwsError::{DateTime, InvalidSiteIdentifier}(String)` | Removed; invalid archive names fail in `Identifier::parse` with `InvalidIdentifier` | #192 |
| (new) | `AwsError::{InvalidIdentifier { name, reason }, InvalidVolumeIndex { index }, HttpClient(Box<dyn Error + Send + Sync>)}` | #192 |
| `aws::realtime::ChunkIterator::try_next()` retrying volume discovery and returning `ExpectedChunkNotFound` when exhausted | Single attempt; `Ok(None)` until available | #192 |
| `aws::realtime::chunk_stream(..)` discarding its startup chunks | Yields the Start chunk (when joining mid-volume) and the latest chunk first | #192 |
| `aws::realtime::Chunk<'a>`, `Chunk::IntermediateOrEnd(Record<'a>)` | `Chunk`, `Chunk::IntermediateOrEnd(Record<'static>)` | #167 |
| `aws::realtime::DownloadedChunk::chunk: Chunk<'static>` | `chunk: Chunk`; struct is `#[non_exhaustive]`, `DownloadedChunk::new` | #167 |
| `aws::realtime::ChunkIteratorInit`, `LatestVolumeResult` | `#[non_exhaustive]` | #167 |
| `aws::realtime::assemble_volume<'a>(impl IntoIterator<Item = Chunk<'a>>)` | `assemble_volume(impl IntoIterator<Item = Chunk>)` | #167 |
| `aws::realtime::chunk_stream(PollConfig)` | `chunk_stream(PollConfig, S) where S: Fn(Duration) -> F, F: Future<Output = ()>`, for example `\|d\| tokio::time::sleep(d)` | #161 |
| `aws::realtime::download_chunk<'a>(&str, &ChunkIdentifier) -> Result<(ChunkIdentifier, Chunk<'a>)>` | `download_chunk(&ChunkIdentifier) -> Result<DownloadedChunk>`; read `d.identifier`, `d.chunk` | #167, #206 |
| `aws::realtime::ChunkIdentifier::new(.., NaiveDateTime, ..)` | `new(.., DateTime<Utc>, ..)`; `naive.and_utc()` converts, and fractional seconds are dropped | #206 |
| `aws::realtime::ChunkIdentifier::date_time_prefix() -> &NaiveDateTime`, `volume() -> &VolumeIndex` | `date_time_prefix() -> DateTime<Utc>`, `volume() -> VolumeIndex` | #206 |
| `aws::realtime::VolumeIndex::as_number() -> usize` | `number() -> usize` | #206 |
| `aws::realtime::ChunkType` | `#[non_exhaustive]` | #206 |
| `aws::realtime::list_chunks_in_volume` failing on a key that does not parse | Skips the key with a `log` warning | #206 |
| `aws::realtime::ChunkIterator::from_chunk(..)` whose `try_next` failed with `FailedToDetermineNextChunk` | The first `try_next` loads the VCP from the current volume's Start chunk, then returns the next chunk | #206 |
| `result::Error::UncompressedData` from `Record::decompress` on an uncompressed record | Removed; the record is returned unchanged | #206 |
| `result::Error::CompressedData` | `Error::CompressedRecord` | #206 |
| `Error::{TruncatedRecord, InvalidRecordSize}`, `AwsError::{InvalidIdentifier, InvalidVolumeIndex, S3GetObject}` | `#[non_exhaustive]` variants: patterns need `..` | #206 |
| (new) | `volume::Record::iter_radials() -> RadialIter`, one `Result<Radial>` per radar data message (feature `model`) | #206 |
| `aws::realtime::ChunkIdentifier::new(String, ..)`, `from_name(String, VolumeIndex, String, ..)` | `new(impl Into<String>, ..)`, `from_name(impl Into<String>, VolumeIndex, impl Into<String>, ..)` | #167 |
| `aws::realtime::ChunkIdentifier::next_chunk(&ElevationChunkMapper)`, `NextChunk::{Sequence, Volume}` | Removed; `ChunkIterator` and `chunk_stream` sequence chunks, and `ChunkIdentifier::with_sequence(usize, ChunkType)` names a sibling chunk | #167 |
| `aws::realtime::ChunkIterator::start_with_policies(&str, RetryPolicy, RetryPolicy)` | `start_with(PollConfig::new(site).with_download_policy(..).with_discovery_policy(..))` | #167 |
| `aws::realtime::ChunkIterator::from_chunk(&str, ChunkIdentifier, RetryPolicy, RetryPolicy)` | `from_chunk(PollConfig, ChunkIdentifier)` | #167 |
| `aws::realtime::ChunkIterator::{download_policy, discovery_policy}()` | `config().download_policy()`, `config().discovery_policy()` | #167 |
| `aws::realtime::ChunkIterator::time_until_next() -> Option<chrono::Duration>` | `-> Option<std::time::Duration>` | #167 |
| `aws::realtime::ChunkIterator::project_remaining_scan() -> Option<ScanTimingProjection>` | `projected_volume_end_time() -> Option<DateTime<Utc>>` | #167 |
| `aws::realtime::ChunkIterator::{timing_stats, timing_stats_mut, elevation_mapper, chunk_metadata, all_chunk_metadata}()` | Removed | #167 |
| `aws::realtime::PollConfig::{site, download_policy, discovery_policy}` fields | Getters of the same names and the existing `with_*` setters | #167 |
| `aws::realtime::RetryPolicy::{max_attempts, initial_delay, backoff_multiplier, max_delay}` fields | Getters of the same names (`std::time::Duration`) | #167 |
| `aws::realtime::RetryPolicy::with_initial_delay(chrono::Duration)`, `with_max_delay(chrono::Duration)` | Take `std::time::Duration` | #167 |
| `aws::realtime::{ChunkTimingModel, ChunkTimingStats, ChunkCharacteristics, ElevationChunkMapper, ChunkMetadata, ChunkProjection, ScanTimingProjection, RetryState}` | Removed; `ChunkIterator` and `chunk_stream` handle sequencing, retries (configured with `RetryPolicy`) and timing | #167 |
| `aws::realtime::{estimate_chunk_availability_time, estimate_chunk_processing_time, project_scan_timing, project_full_scan_timing}` | Removed; `ChunkIterator::next_expected_time()`, `time_until_next()` and `projected_volume_end_time()` | #167 |
| (new) | `volume::File::coverage_pattern_message() -> Result<volume_coverage_pattern::Message<'static>>` | #183 |
| `volume::Record::messages()`, `radials()`, `File::scan()`, `aws::realtime::assemble_volume` skipping undecodable messages | Fail with `Error::Decode` on the first one | #190 |
| `aws::realtime::ChunkIterator::try_next()`, `chunk_stream(..)` at volume rollover: newest chunk of the next volume | The next volume's Start chunk, then each later chunk in sequence order | #183 |
| feature `aws` (with rustls) | `aws` (no TLS backend) and default `aws-rustls` (rustls with aws-lc-rs) | #204 |

</details>

### nexrad-process 1.0.0-rc.1 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.1 | 1.0 | PR |
|---|---|---|
| `ScanProcessor` trait (`name`, `process_scan`) | Removed (no implementations); use a free function or your own trait | #175 |
| `ScanDerivedProduct::compute(&self, &Scan, &[SweepField], &RadarCoordinateSystem, &GeoExtent, (usize, usize))` | Trait removed; inherent `CompositeReflectivity::compute(&self, &[SweepField], &RadarCoordinateSystem, &CartesianGrid)` | #175, #193, #207 |
| `ScanDerivedProduct::name()` | Removed with the trait | #207 |
| `trait SweepProcessor` | `trait SweepProcessor: Send + Sync` | #193 |
| `derived::CompositeReflectivity` (unit struct literal) | `#[non_exhaustive]`; `CompositeReflectivity::new()` or `CompositeReflectivity::default()` | #207 |
| (none) | `CartesianGrid::new(GeoExtent, usize, usize) -> Result<Self>`, `extent()`, `width()`, `height()` | #193 |
| `pipeline::SweepPipeline` | `SweepPipeline` only; adds `len()`, `is_empty()`, `steps()` | #193 |
| `filter::ThresholdFilter::{min, max}` fields | `ThresholdFilter::new(Option<f32>, Option<f32>) -> Result<Self>`, `min()`, `max()` | #175 |
| `filter::MedianFilter::{azimuth_kernel, range_kernel}` fields | `MedianFilter::new(usize, usize) -> Result<Self>`, `azimuth_kernel_radials()`, `range_kernel_gates()` | #175, #193 |
| `filter::GaussianSmooth::{sigma_azimuth, sigma_range}` fields | `GaussianSmooth::new(f32, f32) -> Result<Self>`, `sigma_azimuth_radials()`, `sigma_range_gates()` | #175, #193 |
| `filter::CorrelationCoefficientFilter::new(f32, SweepField) -> Result<Self>` | Same signature; adds `threshold()`, `correlation_coefficient()` | #193 |
| `derived::StormRelativeVelocity::new(f32, f32) -> Result<Self>` | Same signature; adds `storm_direction_degrees()`, `storm_speed_mps()` | #193 |
| `derived::VerticalCrossSection::new(f32, f64, f64, usize, usize)` | `VerticalCrossSection::builder(f64) -> VerticalCrossSectionBuilder`, setters `max_range_km(f64)`, `max_altitude_meters(f64)`, `width(usize)`, `height(usize)`, `build() -> Result<VerticalCrossSection>`; getters for all five | #164, #182, #193 |
| `derived::VerticalCrossSection::compute(&self, &[SweepField])` | `compute(&self, &[SweepField], &RadarCoordinateSystem)` | #188 |
| `detection::StormCell` | `detection::ThresholdRegion`, same methods | #207 |
| `detection::StormCellDetector` | `detection::ThresholdRegionDetector`, same methods | #207 |
| `detection::StormCell::elevation_degrees() -> f32` | `ThresholdRegion::elevation_angle_degrees() -> f32` | #182, #207 |
| `detection::StormCell::max_reflectivity_azimuth_degrees() -> f32` | `ThresholdRegion::max_reflectivity_azimuth_angle_degrees() -> f32` | #182, #207 |
| `detection::StormCell::bounds() -> &StormCellBounds` | `ThresholdRegion::bounds() -> GeoExtent`; `bounds().min.latitude` for `min_latitude()`, and so on | #193, #207 |
| `detection::StormCellBounds` | Removed; use `nexrad_model::geo::GeoExtent` | #193 |
| `detection::StormCellDetector::new(f32, usize) -> Result<Self>` | `ThresholdRegionDetector::new(f32, usize) -> Result<Self>`; adds `reflectivity_threshold_dbz()`, `min_gate_count()` | #193, #207 |
| `filter::{ThresholdFilter, CorrelationCoefficientFilter}` mask gates as `(0.0, GateStatus::NoData)` | Mask gates as `(value, GateStatus::Filtered)` | #187 |
| `result::Error` | `#[non_exhaustive]`, `Clone`, `PartialEq`; new `Model(nexrad_model::Error)`, `GeometryMismatch { parameter }`, `UnexpectedProduct { expected, actual }` | #155, #185, #193 |
| `result::Error::{InvalidParameter, InvalidGeometry, MissingData}(String)` | `InvalidParameter { parameter, reason }`, `InvalidGeometry { parameter, reason }`, `MissingData { what }` | #193 |

</details>

### nexrad-render 1.0.0-rc.4 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `render_radials(&[Radial], Product, &DiscreteColorScale, &RenderOptions) -> Result<RgbaImage>` | Removed; `render_sweep(&SweepField::from_radials(radials, product)?, &scale.into(), &options)?.into_pixels()` | #174, #203 |
| `render_radials_default(&[Radial], Product, &RenderOptions) -> Result<RgbaImage>` | Removed; `render_sweep` with `default_scale(product)` | #174 |
| `default_color_scale(Product) -> ColorScale` | Removed; `default_scale` | #174 |
| `default_scale(Product) -> DiscreteColorScale` | `default_scale(Product) -> ColorScale` | #174 |
| `nws_reflectivity_scale()` | `reflectivity_scale()` | #210 |
| `DiscreteColorScale::new(Vec<ColorScaleLevel>) -> Self`, `ContinuousColorScale::new(Vec<ColorStop>) -> Self` | `-> Result<Self>`; an empty list or a non-finite value is `Error::InvalidColorScale { reason }` | #210 |
| `ColorLookupTable::from_scale(&DiscreteColorScale, f32, f32, usize)`, `from_color_scale(&ColorScale, f32, f32, usize)` | `ColorLookupTable::new(&ColorScale, f32, f32, usize) -> Result<ColorLookupTable>`; convert a `DiscreteColorScale` with `.into()` | #174, #194 |
| `ColorScaleLevel` private fields | `pub value: f32`, `pub color: Color` | #194 |
| `ColorScale`, `Interpolation` | `#[non_exhaustive]`; add a wildcard arm | #207 |
| `Color::to_rgba8` truncating | Rounds to nearest, clamps to `0.0..=1.0` | #194 |
| `RenderOptions::new(usize, usize)`, `size() -> (usize, usize)` | `new(u32, u32)`, `size() -> (u32, u32)` | #174 |
| `RenderOptions::transparent()`, `bilinear()` | Removed; `with_background(Color::TRANSPARENT)`, `with_interpolation(Interpolation::Bilinear)` | #194 |
| `RenderOptions::with_background([u8; 4])`, `background() -> Option<[u8; 4]>` | `with_background(Color)` (`Color::from_rgba8(rgba)` or `rgba.into()`), `background() -> Option<Color>` (`None` when fully transparent) | #194 |
| `RenderResult::new(RgbaImage, RenderMetadata)` | Crate-private | #174 |
| `RenderResult::image() -> &RgbaImage`, `into_image() -> RgbaImage` | `pixels() -> &[u8]`, `into_pixels() -> Vec<u8>` (RGBA8, row-major) with `width()`, `height()`; `into_rgba_image()` with the opt-in `image` feature | #203 |
| `RenderResult::save(path)` (format from the extension) | `save_png(path)`; `write_png(impl Write)` encodes into memory | #203 |
| `nexrad_render::RgbaImage` (re-export of `image::RgbaImage`) | Removed; `image` is not in the API without the opt-in `image` feature | #203 |
| `RenderResult::query_pixel(&SweepField, f64, f64)`, `query_geo(&SweepField, &GeoPoint)` | `RenderMetadata::query_pixel`, `RenderMetadata::query_geo` (`result.metadata().query_pixel(..)`) | #194 |
| `RenderResult::query_polar(&SweepField, f32, f64)` | `RenderMetadata::query_polar(&SweepField, f64, f64)` | #164, #194 |
| `nexrad_render::Product` (re-export) | Removed; `nexrad_model::data::Product` (or `nexrad::model::data::Product`) | #194 |
| `image` dependency with default features | No `image` dependency (PNG through a private encoder); opt-in `image` feature for `into_rgba_image()` | #194, #203 |
| `RenderMetadata::center_pixel() -> (f64, f64)` | `-> Option<(f64, f64)>`, `Some` only for polar renders | #174 |
| `RenderMetadata::{pixels_per_km, max_range_km}() -> f64` | `-> Option<f64>` | #174 |
| `RenderMetadata::{pixel_distance_to_km, km_to_pixel_distance}(f64) -> f64` | `-> Option<f64>` | #174 |
| `RenderMetadata::polar_to_pixel(f32, f64) -> (f64, f64)` | `polar_to_pixel(f64, f64) -> Option<(f64, f64)>` | #164, #174 |
| `RenderMetadata::elevation_degrees() -> Option<f32>` | `elevation_angle_degrees() -> Option<f64>` | #164, #182 |
| `PointQuery` | `#[non_exhaustive]`; read it from `RenderMetadata::query_*`, its `polar` a `PolarPoint` with `azimuth_angle_degrees` and `elevation_angle_degrees` | #182 |
| `result::Error::{NoRadials, ImageSave(String), ProductNotFound}` | `EmptyField`, `Io(std::io::Error)` or `Encode(Box<dyn Error + Send + Sync>)`, removed (only `render_radials` produced it) | #174, #195, #203 |
| `result::Error` | `#[non_exhaustive]`; new `InvalidRange { min, max }`, `InvalidTableSize { size }`, `InvalidColorScale { reason }`, `InvalidExtent`, `MissingCoordinateSystem` | #155, #194, #210 |
| `result::Error`: `UnwindSafe`, `RefUnwindSafe` | Neither (the boxed encoder error in `Encode` is not) | #174, #203 |

</details>

### nexrad 1.0.0-rc.4 to 1.0

<details>
<summary>Show the table</summary>

| 1.0.0-rc.4 | 1.0 | PR |
|---|---|---|
| `extract_field`, `extract_fields`, `extract_first_field` | `Sweep::field`, `Scan::fields`, `Scan::first_field` | #172 |
| `coordinate_system`, `coordinate_system_required` | `Scan::coordinate_system` (`scan.coordinate_system().ok_or(/* your error */)?` for the latter) | #172 |
| `sites`, `site`, `nearest_site(f32, f32)` | `model::meta::registry::{sites, site_by_id, nearest_site(f64, f64)}` | #164, #172 |
| `download(Identifier)` | `download(&Identifier)` | #176 |
| `load(&[u8])` | `load(Vec<u8>)` | #195 |
| `download_latest(&str, NaiveDate)` | `download_latest(&str)`; for a given date, `list_scans` then `download` | #195 |
| `download_at(&str, NaiveDateTime)` | `download_at(&str, DateTime<Utc>)`; `naive.and_utc()` converts | #206 |
| `download_at` miss: `Error::NoDataAvailable` | `Error::NoScanAt { site, datetime: DateTime<Utc> }`; the previous UTC date is searched too | #195, #206 |
| `stream(&str)` | `stream(&str, S) where S: Fn(Duration) -> F, F: Future<Output = ()>` | #161 |
| (new) | `stream_with(PollConfig, S)`, `stream_scans(&str, S)`, `stream_scans_with(PollConfig, S)` | #195 |
| (your own `chrono` dependency) | `nexrad::chrono` (feature `chrono`, implied by `data`) | #176, #204 |
| feature `wasm` | Removed; list the features you use (every feature except `parallel` builds for `wasm32`) | #204 |
| default features `full` | `data`, `aws`, `aws-rustls`, `render` | #204 |
| feature `aws` (with rustls) | `aws` (no TLS backend) and `aws-rustls` (rustls with aws-lc-rs) | #204 |
| `Error::Data(data::Error::Decode(e))`, `Error::Data(data::Error::Model(e))`, `Error::Data(data::Error::Io(e))` | `Error::Decode(e)`, `Error::Model(e)`, `Error::Io(e)` | #176 |
| `Error::MissingSiteMetadata` | Removed (only `coordinate_system_required` produced it) | #172 |
| `Error::NoDataAvailable::date: String` | `date: chrono::NaiveDate`; variant requires `aws` | #176 |
| `Error::{Model, Decode, Data, Render, Process, Io}` `Display` `"<layer> error: <message>"` | `"<layer> error"`; the message comes from `source()` | #176, #195 |
| `Error` | `#[non_exhaustive]` | #155 |
| Features `aws-polling`, `tokio` | Removed; use `aws` | #156, #161 |
| Feature `uom` | Removed; use the unit-suffixed accessors | #202 |

</details>

### 0.x baselines

<details>
<summary>Show the notes</summary>

The [Upgrading from 0.x](#upgrading-from-0x) table covers every public entry point of `nexrad` 0.0.3, `nexrad-model`
0.1.0, `nexrad-decode` 0.1.2 and `nexrad-data` 0.2.0. Beyond it, the 0.x diffs differ from the release-candidate diffs
above only in:

- `nexrad` 0.0.3: every item of the `model` module (`DataFile`, `Message31`, `Message31Header`, `MessageHeader`,
  `DataMoment`, `GenericData`, `DataBlockHeader`, `VolumeData`, `ElevationData`, `RadialData`, `VolumeHeaderRecord`)
  is removed; the table names their successors.
- `nexrad-decode` 0.1.2: the message structs gained a lifetime (`Message<'a>`, `Header<'a>`, ...) and lost their public
  fields to the accessors in the [nexrad-decode tables](#nexrad-decode-100-rc3-to-10). Legacy digital radar data,
  performance/maintenance, console, control command, clutter map, censor zone, request, loopback, adaptation, PRF and
  log messages decode into typed `MessageContents` variants instead of `Other`.
- `nexrad-data` 0.2.0: the realtime timing and iterator API (`ChunkIterator`, `chunk_stream`, `PollConfig`,
  `RetryPolicy`, `DownloadedChunk`, `assemble_volume`) is new, and `File::compressed`/`decompress` handle gzip-wrapped
  volumes.

</details>

### How these tables were produced

From [`cargo public-api`](https://github.com/cargo-public-api/cargo-public-api) diffs of each crate against its
published baseline, without trait-implementation noise, cross-checked against the CHANGELOGs and the pull requests they
cite. To see the full diff for your version, run from a checkout (rustdoc JSON needs the nightly pinned in
`xtask/src/public_api.rs`):

```sh
cargo install cargo-public-api
RUSTUP_TOOLCHAIN=nightly cargo public-api -p nexrad-data --all-features --simplified diff 1.0.0-rc.5
```

The output's "Removed items", "Changed items" and "Added items" sections show a changed path or signature as a removal
plus an addition.
