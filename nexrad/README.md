# nexrad

[![Crates.io](https://img.shields.io/crates/v/nexrad)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/nexrad)](https://github.com/danielway/nexrad#installation)
[![License](https://img.shields.io/crates/l/nexrad)](https://github.com/danielway/nexrad/blob/main/LICENSE)

Load, download, decode, process and render NEXRAD weather radar data.

![Reflectivity from the lowest tilt of a KDMX volume](https://raw.githubusercontent.com/danielway/nexrad/nexrad@1.0.0/docs/images/reflectivity.png)

*Lowest-tilt reflectivity from KDMX (Des Moines, Iowa), 5 March 2022 23:23 UTC, drawn by this crate's `render` feature.*

This is the main entry point for the NEXRAD library suite. It re-exports the underlying crates
(`nexrad-model`, `nexrad-decode`, `nexrad-data`, `nexrad-process`, `nexrad-render`) and provides
top-level convenience functions for common tasks.

## Scope

The nexrad crates read NEXRAD Level II data in the Archive II format (ICD 2620010) and the RDA/RPG messages it carries
(ICD 2620002). WSR-88D volumes decode, and so do TDWR Level II volumes. The intended scope for 1.0 is the whole archive:
volumes with legacy message 1 radials, volumes from before 2008 that carry no VCP message, and gzip-compressed volumes
back to 1991 are all inputs. `_MDM` objects (Model Data, message type 29) and archive tarballs are not read, and Level
III products are out of scope.

## Quick Start

```rust,no_run
fn main() -> nexrad::Result<()> {
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
    Ok(())
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

`nexrad::chrono` re-exports `chrono` (with the `chrono` feature, implied by `data`) because
`list_scans` takes a `NaiveDate` (a UTC calendar date) and `download_at` a `DateTime<Utc>`, so you
don't need a separate `chrono` dependency to call them. The real-time streams implement `futures_core::Stream`; add
`futures-util` (or `futures`) to your own dependencies for `StreamExt::next`.

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

The default features, `data`, `aws`, `aws-rustls` and `render`, load local files, download from
AWS and render images. `full` enables every feature except `image`. Each feature also enables the
features it depends on, so `data` alone is enough for local file loading.

| Feature | Description | Implies |
|---------|-------------|---------|
| `model` | Core data types | |
| `decode` | Protocol decoding | `model` |
| `data` | Local file I/O (`load`, `load_file`) | `decode`, `model`, `chrono` |
| `render` | Image rendering | `model` |
| `process` | Processing algorithms | `model` |
| `aws` | AWS S3 archive downloads and real-time streaming, without a TLS backend | `data` |
| `aws-rustls` | `aws` with rustls and the `aws-lc-rs` crypto provider as the TLS backend | `aws` |
| `parallel` | Parallel decompression (requires rayon) | `data` |
| `serde` | Serialization support | |
| `chrono` | DateTime type support | |
| `image` | `RenderResult::into_rgba_image` interop (opt-in, outside the semver promise) | `render` |
| `full` | Every feature above except `image` | all of them except `image` |

Every feature except `parallel` builds for `wasm32-unknown-unknown`.

Accessors return plain numbers with the unit in the name. Unit-typed accessors (for example with the `uom` crate) may
be offered later as a separate crate or extension trait.

The `image` feature is not part of `full`. It tracks the `image` crate's own 0.x releases and is
excluded from the semver promise: a minor or patch release may move it to a newer, incompatible `image` version.
Without it, rendered images are plain RGBA8 pixel buffers with `write_png` and `save_png`.

For local file processing without rendering or networking:

```toml
nexrad = { version = "1.0", default-features = false, features = ["data"] }
```

### TLS

On native targets, `aws` alone compiles but has no TLS backend, so every HTTPS request fails at run
time with reqwest's `invalid URL, scheme is not http` connect error. A TLS feature of `reqwest` 0.13
(for example `native-tls`) enabled by any other crate in the build also provides one, since Cargo
unifies features. `wasm32` needs no TLS backend because the browser performs TLS. `aws-rustls` (on by default) adds rustls with the `aws-lc-rs` crypto provider,
which compiles C code and needs a C compiler and, on Windows x86 targets, NASM. To use a different
TLS backend, disable the default features, enable `aws`, and enable a TLS feature of `reqwest` 0.13
in your own manifest.

## WASM Support

Every feature except `parallel` builds for `wasm32-unknown-unknown`. On `wasm32`, `aws` sends
requests through the JavaScript `fetch` API and needs no TLS feature:

```toml
nexrad = { version = "1.0", default-features = false, features = [
    "data", "render", "process", "aws", "serde", "chrono",
] }
```

The `parallel` feature requires a native thread pool (rayon). Real-time streaming is part of `aws`
and is runtime-agnostic, so it works on WASM with any sleep future.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0. Each
re-exported crate has its own CHANGELOG. To upgrade from 0.x or a release candidate, see
[Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).
