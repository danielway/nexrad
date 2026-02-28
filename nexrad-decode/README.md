# NEXRAD Decode

[![Crates.io](https://img.shields.io/crates/v/nexrad-decode)](https://crates.io/crates/nexrad-decode)
[![Docs.rs](https://docs.rs/nexrad-decode/badge.svg)](https://docs.rs/nexrad-decode)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are in accordance with
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002AA".

## Usage

The main entry point is `messages::decode_messages`, which parses decompressed binary data into structured messages:

```rust,ignore
use nexrad_decode::messages::{decode_messages, MessageContents};

let messages = decode_messages(&decompressed_data)?;

for message in &messages {
    match message.contents() {
        MessageContents::DigitalRadarData(data) => {
            println!("Azimuth: {:.1}°, Elevation: {:.1}°",
                data.azimuth_angle(), data.elevation_angle());
        }
        MessageContents::VolumeCoveragePattern(vcp) => {
            println!("VCP: {}", vcp.pattern_number());
        }
        _ => {}
    }
}
```

For single-message parsing (e.g., streaming), use `decode_message` which returns the remaining bytes:

```rust,ignore
use nexrad_decode::messages::decode_message;

let (remaining, message) = decode_message(message_bytes)?;
```

## Message Types

The decoder handles several message types defined in the ICD:

| Type | Name | Contents |
|------|------|----------|
| 31 | Digital Radar Data | Primary radar measurements (reflectivity, velocity, etc.) |
| 2 | RDA Status Data | Radar status, alarms, and operational parameters |
| 5 | Volume Coverage Pattern | Scanning strategy (elevation cuts, waveforms) |
| 15 | Clutter Filter Map | Clutter suppression configuration |

## Modules

- `messages` — Message parsing and type definitions
- `summarize` — Utilities for summarizing message collections (counts, types, elevation coverage)
- `result` — Error and result types

## Features

- `nexrad-model`: Provides mappings to the common `nexrad-model` types for higher-level use.
- `uom`: Use the `uom` crate for type-safe units of measure.

## Crate Boundaries

This crate provides **binary protocol parsing** only:

- No I/O operations (operates on byte slices provided by the caller)
- No file or network access
- No decompression (the caller must decompress LDM records first)

For file I/O and decompression, see [`nexrad-data`](https://crates.io/crates/nexrad-data). For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
