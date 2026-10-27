# NEXRAD

[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

A suite of tools for working with NEXRAD WSR-88D weather radar data.

## `nexrad`

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)

Ergonomic APIs for accessing, decoding, and processing NEXRAD weather radar data.

## `nexrad-model`

[![Crate](https://img.shields.io/crates/v/nexrad-model.svg)](https://crates.io/crates/nexrad-model)
[![Docs.rs](https://docs.rs/nexrad-model/badge.svg)](https://docs.rs/nexrad-model)

A common model for representing NEXRAD weather radar data. Provides an ergonomic API which is documented for an audience
who is not necessarily familiar with the NOAA Archive II format.

## `nexrad-decode`

[![Crate](https://img.shields.io/crates/v/nexrad-decode.svg)](https://crates.io/crates/nexrad-decode)
[![Docs.rs](https://docs.rs/nexrad-decode/badge.svg)](https://docs.rs/nexrad-decode)

Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are in accordance with
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002AA".

## `nexrad-data`

[![Crate](https://img.shields.io/crates/v/nexrad-data.svg)](https://crates.io/crates/nexrad-data)
[![Docs.rs](https://docs.rs/nexrad-data/badge.svg)](https://docs.rs/nexrad-data)

Download and processing functions for NEXRAD weather radar data.

## `nexrad-process`

[![Crate](https://img.shields.io/crates/v/nexrad-process.svg)](https://crates.io/crates/nexrad-process)
[![Docs.rs](https://docs.rs/nexrad-process/badge.svg)](https://docs.rs/nexrad-process)

Processing algorithms for NEXRAD weather radar data.

## `nexrad-render`

[![Crate](https://img.shields.io/crates/v/nexrad-render.svg)](https://crates.io/crates/nexrad-render)
[![Docs.rs](https://docs.rs/nexrad-render/badge.svg)](https://docs.rs/nexrad-render)

Functions for rendering NEXRAD weather radar data into visual images.

## `nexrad-inspector`

Interactive TUI for inspecting NEXRAD Archive II volume files. Browse local files or download directly
from AWS, decompress LDM records, and inspect individual radar messages with hex and parsed views.

This can be run from the repository with:

```bash
cargo run -p nexrad-inspector
```

## Getting Started

### Installation

Add `nexrad` to your `Cargo.toml`:

```toml
[dependencies]
nexrad = "1.0"
```

The default features cover local file loading, AWS downloads and real-time streaming, and rendering. Processing
(`process`), serialization (`serde`), units of measure (`uom`), parallel decoding (`parallel`) and `image` crate
interop (`image`) are opt-in, and `full` enables every feature except `image`.

**Common configurations:**

```toml
# Minimal - local file processing only (no rendering, no networking)
nexrad = { version = "1.0", default-features = false, features = ["data"] }

# AWS S3 downloads without rendering
nexrad = { version = "1.0", default-features = false, features = ["aws-rustls"] }
tokio = { version = "1", features = ["full"] }

# Every feature except `image` (native only)
nexrad = { version = "1.0", features = ["full"] }
```

**Minimum supported Rust version:** 1.88. The MSRV is only raised in minor releases, never in patch releases.

### Quick Example: Load and Inspect a Scan

```rust,no_run
fn main() -> nexrad::Result<()> {
    // Load from a local Archive II file
    let scan = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;

    println!("{}", scan.coverage_pattern_number());
    println!("Sweeps: {}", scan.sweeps().len());

    // Iterate through sweeps and radials
    for sweep in scan.sweeps() {
        println!("Elevation {}: {} radials",
            sweep.elevation_number(),
            sweep.radials().len());

        for radial in sweep.radials() {
            if let Some(reflectivity) = radial.reflectivity() {
                println!("  {} reflectivity gates", reflectivity.gate_count());
            }
        }
    }

    Ok(())
}
```

### Download from AWS

With the `aws` feature enabled, download radar data directly from the NEXRAD archive:

```rust,ignore
use nexrad::chrono::NaiveDate;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();

    // List available scans
    let scans = nexrad::list_scans("KTLX", date).await?;
    println!("Found {} scans", scans.len());

    // Download the last scan of that day
    if let Some(id) = scans.last() {
        let scan = nexrad::download(id).await?;
        println!("{}", scan.coverage_pattern_number());
    }

    // Download the newest scan available now
    let scan = nexrad::download_latest("KTLX").await?;
    println!("{}", scan.coverage_pattern_number());

    Ok(())
}
```

### Render Radar Images

With the `render` feature, create PNG images from radar data:

```rust,ignore
use nexrad::model::data::Product;
use nexrad::render::{default_scale, render_sweep, RenderOptions};

fn main() -> nexrad::Result<()> {
    let scan = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
    let sweep = scan.sweeps().first().unwrap();
    let field = sweep.field(Product::Reflectivity).unwrap();

    let options = RenderOptions::new(1024, 1024);
    let color_scale = default_scale(Product::Reflectivity);

    let result = render_sweep(&field, &color_scale, &options)?;
    result.save_png("reflectivity.png")?;
    Ok(())
}
```

### Feature Reference

The default features of the `nexrad` crate are `data`, `aws`, `aws-rustls` and `render`. Each feature also enables
the features listed under "Implies".

| Feature | Description | Implies | Dependencies |
|---------|-------------|---------|--------------|
| `model` | Core data types (Scan, Sweep, Radial) | | Pure Rust |
| `decode` | Binary protocol decoding | `model` | chrono, zerocopy |
| `data` | Local file I/O | `decode`, `model`, `chrono` | bzip2 |
| `render` | Image rendering | `model` | png |
| `process` | Processing algorithms | `model` | nexrad-model |
| `aws` | AWS S3 downloads and real-time streaming, without a TLS backend | `data` | reqwest, futures-util |
| `aws-rustls` | `aws` with rustls (aws-lc-rs provider) as the TLS backend | `aws` | rustls, aws-lc-rs |
| `parallel` | Parallel decompression | `data` | rayon |
| `serde` | Serialization support | | serde |
| `uom` | Type-safe units of measure | | uom |
| `chrono` | DateTime type support | | chrono |
| `image` | `RenderResult::into_rgba_image` (opt-in, not in `full`, outside the semver promise) | `render` | image |
| `full` | Every feature above except `image` | All above except `image` | All above |

On native targets `aws` needs a TLS backend to reach the HTTPS endpoints; `aws-rustls` provides one and needs a C
compiler and, on Windows x86 targets, NASM.

### WASM Support

Every feature except `parallel` (which needs threads) builds for `wasm32-unknown-unknown`. On `wasm32`, `aws` uses the
JavaScript `fetch` API and needs no TLS feature:

```toml
nexrad = { version = "1.0", default-features = false, features = [
    "data", "render", "process", "aws", "serde", "uom", "chrono",
] }
```

Real-time streaming (`nexrad::stream`, `nexrad::stream_scans`) is part of `aws` and works on WASM: it takes a caller-supplied sleep
function instead of depending on a specific async runtime.

### Examples

Run the included examples:

```bash
# Decode and summarize a volume file
cargo run -p nexrad --example decode_summary -- path/to/volume.ar2v

# Download latest data from AWS
cargo run -p nexrad --example download_latest -- KTLX

# Render reflectivity image
cargo run -p nexrad --example render_reflectivity -- path/to/volume.ar2v output.png

# Filter a sweep, compute composite reflectivity, and render both
cargo run -p nexrad --example processing_pipeline --features process -- path/to/volume.ar2v

# Render every product and processing step for a full scan into renders/
cargo run -p nexrad --example showcase --features process --release
```

### Lower-Level APIs

For advanced use cases, you can use the sub-crates directly:

- **`nexrad-decode`** - Parse individual NEXRAD messages from byte slices
- **`nexrad-data`** - Handle Archive II file decompression and AWS S3 access
- **`nexrad-model`** - Work directly with domain types
- **`nexrad-process`** - Filter sweeps and compute derived products
- **`nexrad-render`** - Render fields to images

See each crate's documentation for detailed API information, and [docs/architecture.md](docs/architecture.md) for how
the crates fit together and the API stability rules they follow.

### Changelogs

Each library crate records its changes in its own CHANGELOG:
[`nexrad`](nexrad/CHANGELOG.md), [`nexrad-model`](nexrad-model/CHANGELOG.md),
[`nexrad-decode`](nexrad-decode/CHANGELOG.md), [`nexrad-data`](nexrad-data/CHANGELOG.md),
[`nexrad-process`](nexrad-process/CHANGELOG.md) and [`nexrad-render`](nexrad-render/CHANGELOG.md).

Upgrading from a 0.x release or a `1.0.0-rc` release candidate? [Migrating to 1.0](docs/migrating-to-1.0.md) walks
through the changes crate by crate, with before-and-after examples.

## Maintaining the Radar Site Registry

The checked-in NEXRAD site registry is audited against NOAA's radar site catalog.
The normal workspace test suite checks the registry against a checked-in snapshot
offline, and a weekly workflow compares that snapshot with NOAA's current data.
To fetch NOAA data and refresh the snapshot explicitly, run:

```bash
cargo run -p xtask -- update-radar-sites
```

See [Radar site registry maintenance](docs/radar-site-registry.md) for the audit
policy, failure behavior, and complete update workflow.

## Acknowledgements

I consulted the following resources when developing this library:

NOAA NCEI, NEXRAD System and Product Description with Access Information:
https://www.ncei.noaa.gov/products/radar/next-generation-weather-radar

NOAA NWS, Radar Operations Center, NEXRAD WSR-88D Level II Data Information:
https://www.roc.noaa.gov/wsr88d/level_ii/level2info.aspx

NOAA NWS, Radar Operations Center, NEXRAD WSR-88D Interface Control Documents:
https://www.roc.noaa.gov/wsr88d/BuildInfo/Files.aspx

NASA TRMM, Radar Software Library:
https://trmm-fc.gsfc.nasa.gov/trmm_gv/software/rsl/

Brian Wigginton, a Go implementation of NEXRAD Level II decoding:
https://github.com/bwiggs/go-nexrad
