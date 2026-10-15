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
| `load(data)` | Load from raw Archive II bytes, taking ownership of the `Vec<u8>` |
| `load_file(path)` | Load from a file path |

### AWS Downloads (requires `aws` feature)

| Function | Description |
|----------|-------------|
| `list_scans(site, date)` | List available scans for a site and date |
| `download(&identifier)` | Download a specific scan by archive identifier (taken by reference, as returned by `list_scans`) |
| `download_latest(site)` | Download the newest archived scan (for the last scan of a given date, `list_scans` then `download`) |
| `download_at(site, datetime)` | Download the scan overlapping a specific UTC time, which may have started on the previous date |

### Real-Time Streaming (requires `aws` feature)

| Function | Description |
|----------|-------------|
| `stream(site, sleep)` | Stream live radar data chunks as a `Stream`, waiting via the supplied sleep function (e.g. `\|d\| tokio::time::sleep(d)`) |
| `stream_with(config, sleep)` | `stream` with a `PollConfig` that sets the retry policies |
| `stream_scans(site, sleep)` | Stream one `Scan` per complete live volume |
| `stream_scans_with(config, sleep)` | `stream_scans` with a `PollConfig` |

## Re-exported Crates

Access sub-crate APIs through module re-exports:

- `nexrad::model` — Core data types (`Scan`, `Sweep`, `Radial`, `SweepField`, `Site`)
- `nexrad::decode` — Binary protocol decoding (`decode_messages`, message types)
- `nexrad::data` — File I/O and AWS S3 (`volume::File`, `aws::archive`, `aws::realtime`)
- `nexrad::process` — Processing algorithms (`SweepPipeline`, filters, derived products)
- `nexrad::render` — Visualization (`render_sweep`, color scales, `RenderOptions`)

`nexrad::chrono` re-exports `chrono` (with the `chrono` feature, implied by `aws`) because
`list_scans` and `download_at` take `NaiveDate`/`NaiveDateTime`, so you don't need a separate
`chrono` dependency to call them. `nexrad::futures_util` re-exports `futures_util` (with `aws`)
for `StreamExt::next` on the real-time streams.

Helpers that only need model types live on those types: `sweep.field(product)`,
`scan.fields(product)` and `scan.first_field(product)` extract `SweepField`s,
`scan.coordinate_system()` builds a `RadarCoordinateSystem` from the scan's site, and
`nexrad::model::meta::registry` provides `sites()`, `site_by_id(id)` and
`nearest_site(lat, lon)` for the static site registry.

## Error Handling

All top-level functions return `nexrad::Result<T>`, and every sub-crate error converts into
`nexrad::Error` with `?`. Each variant names the layer an error came from: decode, model and
I/O errors raised inside `nexrad-data` surface as `Error::Decode`, `Error::Model` and
`Error::Io`, while `Error::Data` is reserved for data-access failures such as decompression,
archive layout and AWS errors. With `aws`, `Error::NoDataAvailable` reports that
`download_latest` found no archive volume for the site, and `Error::NoScanAt` that no volume
overlaps the datetime passed to `download_at`. Wrapping variants name only the layer in their
`Display` text; the wrapped error is their `source()`.

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

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0. Each
re-exported crate has its own CHANGELOG. To upgrade from 0.x or a release candidate, see
[Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).
