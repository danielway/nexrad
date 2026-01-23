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
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002W".

## `nexrad-data`

[![Crate](https://img.shields.io/crates/v/nexrad-data.svg)](https://crates.io/crates/nexrad-data)
[![Docs.rs](https://docs.rs/nexrad-data/badge.svg)](https://docs.rs/nexrad-data)

Download and processing functions for NEXRAD weather radar data.

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

This enables the default features (`model`, `decode`, `data`, `render`) for local file processing and visualization.

**Common configurations:**

```toml
# Minimal - local file processing only (no rendering)
nexrad = { version = "1.0", default-features = false, features = ["model", "decode", "data"] }

# With AWS S3 downloads
nexrad = { version = "1.0", features = ["aws"] }
tokio = { version = "1", features = ["full"] }

# Full feature set
nexrad = { version = "1.0", features = ["full"] }
```

### Quick Example: Load and Inspect a Volume

```rust,no_run
use nexrad;

fn main() -> nexrad::Result<()> {
    // Load from a local Archive II file
    let volume = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;

    println!("VCP: {}", volume.coverage_pattern_number());
    println!("Sweeps: {}", volume.sweeps().len());

    // Iterate through sweeps and radials
    for sweep in volume.sweeps() {
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
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();

    // List available volumes
    let volumes = nexrad::list_volumes("KTLX", date).await?;
    println!("Found {} volumes", volumes.len());

    // Download the latest volume for the day
    let volume = nexrad::download_latest("KTLX", date).await?;
    println!("VCP: {}", volume.coverage_pattern_number());

    Ok(())
}
```

### Render Radar Images

With the `render` feature, create PNG images from radar data:

```rust,ignore
use nexrad::render::{get_nws_reflectivity_scale, render_radials, Product};
use piet_common::Device;

fn main() -> nexrad::Result<()> {
    let volume = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
    let sweep = volume.sweeps().first().unwrap();

    let mut device = Device::new().unwrap();
    let color_scale = get_nws_reflectivity_scale();

    let image = render_radials(
        &mut device,
        sweep.radials(),
        Product::Reflectivity,
        &color_scale,
        (1024, 1024),
    )?;

    image.save_to_file("reflectivity.png").unwrap();
    Ok(())
}
```

**Note:** The `render` feature requires system graphics libraries:
- **Debian/Ubuntu:** `apt install libcairo2-dev libpango1.0-dev libglib2.0-dev`
- **macOS:** `brew install cairo pango`

### Feature Reference

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `model` | Core data types (Scan, Sweep, Radial) | Pure Rust |
| `decode` | Binary protocol decoding | chrono, zerocopy |
| `data` | Local file I/O | bzip2 |
| `render` | Image rendering | piet, cairo (system) |
| `aws` | AWS S3 downloads | reqwest |
| `parallel` | Parallel decompression | rayon |
| `serde` | Serialization support | serde |
| `uom` | Type-safe units of measure | uom |
| `chrono` | DateTime type support | chrono |
| `aws-polling` | Real-time polling | tokio |
| `wasm` | All WASM-compatible features | (see below) |
| `full` | All features (native only) | All above |

### WASM Support

The `wasm` feature enables all WASM-compatible functionality:

```toml
nexrad = { version = "1.0", default-features = false, features = ["wasm"] }
```

This includes: `model`, `decode`, `data`, `render`, `aws`, `serde`, `uom`, and `chrono`.

**Not WASM-compatible:**
- `aws-polling` - requires tokio runtime
- `parallel` - requires threads (rayon)
- `full` - includes `aws-polling` and `parallel`

### Examples

Run the included examples:

```bash
# Decode and summarize a volume file
cargo run -p nexrad --example decode_summary -- path/to/volume.ar2v

# Download latest data from AWS
cargo run -p nexrad --example download_latest --features aws -- KTLX 2023-05-20

# Render reflectivity image
cargo run -p nexrad --example render_reflectivity --features render -- path/to/volume.ar2v output.png
```

### Lower-Level APIs

For advanced use cases, you can use the sub-crates directly:

- **`nexrad-decode`** - Parse individual NEXRAD messages from byte slices
- **`nexrad-data`** - Handle Archive II file decompression and AWS S3 access
- **`nexrad-model`** - Work directly with domain types

See each crate's documentation for detailed API information.

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
