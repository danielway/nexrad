# NEXRAD Decode

[![Crates.io](https://img.shields.io/crates/v/nexrad-decode)](https://crates.io/crates/nexrad-decode)
[![Docs.rs](https://docs.rs/nexrad-decode/badge.svg)](https://docs.rs/nexrad-decode)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are in accordance with
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002AA".

## Usage

The main entry point is `decode_messages`, which parses decompressed binary data into structured messages. Decoding is
strict: it returns the error for the first message that cannot be decoded. `iter_messages` yields one `Result` per
message instead, for callers that want the messages before a failure.

```rust,ignore
use nexrad_decode::{decode_messages, MessageContents};

let messages = decode_messages(&decompressed_data)?;

for message in &messages {
    match message.contents() {
        MessageContents::DigitalRadarData(data) => {
            println!("Azimuth: {:.1}°, Elevation: {:.1}°",
                data.header().azimuth_angle_degrees(), data.header().elevation_angle_degrees());
        }
        MessageContents::RdaVolumeCoveragePattern(vcp) => {
            println!("VCP: {}", vcp.header().pattern_number());
        }
        _ => {}
    }
}
```

## Building a Scan

With the `model` feature, `model::ScanBuilder` assembles decoded messages into a `nexrad_model::data::Scan`.
Radar data messages (types 1 and 31) become radials, the first volume coverage pattern message (type 5) becomes the
scan's coverage pattern, and the first volume data block identifies the site:

```rust,ignore
use nexrad_decode::messages::decode_messages;
use nexrad_decode::model::ScanBuilder;

let mut builder = ScanBuilder::new();
builder.add_messages(decode_messages(&decompressed_data)?)?;
let scan = builder.build()?;
```

Builders filled independently (for example, one per record decoded in parallel) can be combined in order with
`ScanBuilder::merge`.

## Message Types

`decode_messages` returns each message's contents as a `MessageContents` variant:

| Type | Name | `MessageContents` variant |
|------|------|---------------------------|
| 1 | Digital Radar Data (legacy) | `DigitalRadarDataLegacy` |
| 2 | RDA Status Data | `RdaStatusData` |
| 3 | Performance/Maintenance Data | `PerformanceMaintenanceData` |
| 4, 10 | Console Message | `ConsoleMessage` |
| 5 | Volume Coverage Pattern | `VolumeCoveragePattern` |
| 6 | RDA Control Commands | `RdaControlCommands` |
| 8 | Clutter Censor Zones | `ClutterCensorZones` |
| 9 | Request for Data | `RequestForData` |
| 11, 12 | Loopback Test | `LoopbackTest` |
| 13 | Clutter Filter Bypass Map | `ClutterFilterBypassMap` |
| 15 | Clutter Filter Map | `ClutterFilterMap` |
| 18 | RDA Adaptation Data | `RdaAdaptationData` |
| 31 | Digital Radar Data (generic format) | `DigitalRadarData` |
| 32 | RDA PRF Data | `RdaPrfData` |
| 33 | RDA Log Data | `RdaLogData` |

Other message types (including reserved and unknown type numbers) decode as `MessageContents::Other`, which carries
the message type and the undecoded payload. A clutter filter map (type 15) from a legacy (pre-ORDA) RDA uses a
different layout and also decodes as `Other`. `MessageContents`, `MessageType` and `Error` are `#[non_exhaustive]`.

## Accessor Naming

Accessors on decoded messages are named after the ICD field: `<stem>_raw()` returns the wire value when its encoding
differs from the decoded value, a decoded number carries its unit (`azimuth_angle_degrees()`,
`unambiguous_range_km()`), a decoded enum or struct uses the bare stem (`radial_status()`), and with the `uom` feature
the bare stem returns a `uom` quantity (`azimuth_angle()`). The crate documentation describes the convention in full.

## Modules

- `messages` — Message parsing and type definitions
- `model` — `ScanBuilder` and conversion to `nexrad-model` types (requires the `model` feature)
- `result` — Error and result types (`Error` and `Result` are also re-exported at the crate root)

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `model` | Yes | Conversion of decoded messages to `nexrad-model` types (`ScanBuilder`, `into_radial`, `to_model`, `site`) |
| `uom` | No | Type-safe units of measure via the `uom` crate |

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.
To upgrade from 0.x or a release candidate, see [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).

## Crate Boundaries

This crate provides **binary protocol parsing** only:

- No I/O operations (operates on byte slices provided by the caller)
- No file or network access
- No decompression (the caller must decompress LDM records first)

For file I/O and decompression, see [`nexrad-data`](https://crates.io/crates/nexrad-data). For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
