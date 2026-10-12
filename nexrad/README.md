# NEXRAD

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Ergonomic APIs for accessing, decoding, processing, and rendering NEXRAD weather radar data.

This is the main entry point for the NEXRAD library suite. It re-exports the underlying crates
(`nexrad-model`, `nexrad-decode`, `nexrad-data`, `nexrad-process`, `nexrad-render`) and provides
top-level convenience functions for common tasks.

## Quick Start

```rust,ignore
use nexrad::model::data::DataMoment; // provides gate_count()

// Load a local Archive II file
let scan = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
println!("{}, {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());

// Iterate through sweeps and radials
for sweep in scan.sweeps() {
    for radial in sweep.radials() {
        if let Some(reflectivity) = radial.reflectivity() {
            println!("{} gates", reflectivity.gate_count());
        }
    }
}
```

## Top-Level Functions

### Loading

| Function | Description |
|----------|-------------|
| `load(data)` | Load from raw Archive II bytes |
| `load_file(path)` | Load from a file path |

### AWS Downloads (requires `aws` feature)

| Function | Description |
|----------|-------------|
| `list_scans(site, date)` | List available scans for a site and date |
| `download(identifier)` | Download a specific scan by archive identifier |
| `download_latest(site, date)` | Download the most recent scan for a date |
| `download_at(site, datetime)` | Download the scan overlapping a specific time |

### Real-Time Streaming (requires `aws` feature)

| Function | Description |
|----------|-------------|
| `stream(site, sleep)` | Stream live radar data chunks as a `Stream`, waiting via the supplied sleep function (e.g. `\|d\| tokio::time::sleep(d)`) |

### Field Extraction

| Function | Description |
|----------|-------------|
| `extract_field(sweep, product)` | Extract a `SweepField` from one sweep |
| `extract_fields(scan, product)` | Extract `SweepField`s from all sweeps |
| `extract_first_field(scan, product)` | Find the first sweep with data for a product |

### Coordinate System

| Function | Description |
|----------|-------------|
| `coordinate_system(scan)` | Create a `RadarCoordinateSystem` from scan metadata |
| `coordinate_system_required(scan)` | Same, but returns an error if metadata is missing |

### Site Registry

| Function | Description |
|----------|-------------|
| `sites()` | All NEXRAD radar sites |
| `site(id)` | Look up a site by ICAO identifier |
| `nearest_site(lat, lon)` | Find the nearest site to a location |

## Re-exported Crates

Access sub-crate APIs through module re-exports:

- `nexrad::model` — Core data types (`Scan`, `Sweep`, `Radial`, `SweepField`, `Site`)
- `nexrad::decode` — Binary protocol decoding (`decode_messages`, message types)
- `nexrad::data` — File I/O and AWS S3 (`volume::File`, `aws::archive`, `aws::realtime`)
- `nexrad::process` — Processing algorithms (`SweepPipeline`, filters, derived products)
- `nexrad::render` — Visualization (`render_sweep`, color scales, `RenderOptions`)

## Features

All features are enabled by default through `full`. Each feature also enables the features it
depends on, so `data` alone is enough for local file loading.

| Feature | Description | Implies | WASM |
|---------|-------------|---------|------|
| `model` | Core data types | | Yes |
| `decode` | Protocol decoding | `model` | Yes |
| `data` | Local file I/O (`load`, `load_file`) | `decode`, `model` | Yes |
| `render` | Image rendering | `model` | Yes |
| `process` | Processing algorithms | `model` | Yes |
| `aws` | AWS S3 archive downloads and real-time streaming | `data`, `chrono` | Yes |
| `serde` | Serialization support | | Yes |
| `uom` | Type-safe units of measure | | Yes |
| `chrono` | DateTime type support | | Yes |
| `parallel` | Parallel decompression (requires rayon) | `data` | No |
| `wasm` | All WASM-compatible features | all features marked Yes | Yes |
| `full` | All features (default) | `wasm`, `parallel` | No |

For local file processing without rendering or networking:

```toml
nexrad = { version = "1.0", default-features = false, features = ["data"] }
```

## WASM Support

For WebAssembly targets, use the `wasm` feature which enables all WASM-compatible functionality:

```toml
nexrad = { version = "1.0", default-features = false, features = ["wasm"] }
```

The `parallel` feature requires a native thread pool (rayon) and is not WASM-compatible. Real-time
streaming is part of `aws` and is runtime-agnostic, so it works on WASM with any sleep future.
