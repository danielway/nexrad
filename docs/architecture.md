# Architecture

This document describes the design philosophy and structure of the NEXRAD Rust library suite.

## Mono-Repo Structure

The project is organized as a Cargo workspace with seven crates. Each crate has a single, well-defined responsibility. Users can depend on individual crates for fine-grained control or use the `nexrad` facade crate for an ergonomic all-in-one API.

```
nexrad-model          Pure data types (no I/O, no parsing)
    │
    ├── nexrad-decode  Binary protocol parsing (byte slices in, structs out)
    ├── nexrad-process Processing algorithms (filters, derived products)
    └── nexrad-render  Visualization (data in, images out)
            │
        nexrad-data    I/O boundary: file handling, decompression, AWS S3
            │
          nexrad        Facade: re-exports + ergonomic top-level functions
            │
        nexrad-inspector  Interactive TUI (binary, not a library)
```

## Design Principles

### Separation of Concerns

Each library crate owns exactly one concern:

| Crate | Owns | Does NOT do |
|-------|------|-------------|
| `nexrad-model` | Domain types and transformations | I/O, parsing, rendering |
| `nexrad-decode` | Binary protocol parsing per NOAA ICD | File I/O, network access |
| `nexrad-data` | File handling, decompression, AWS S3 | Rendering, processing |
| `nexrad-process` | Filtering, smoothing, derived products | I/O, rendering |
| `nexrad-render` | Color mapping, image generation | Data access, parsing |
| `nexrad` | Ergonomic facade, unified error types | New functionality |

This separation means you can use `nexrad-model` and `nexrad-render` in a WASM application without pulling in file I/O or native networking code.

### Why nexrad-model and nexrad-decode Are Separate

The data model (`nexrad-model`) and the binary parser (`nexrad-decode`) are intentionally separate crates:

- **nexrad-model** defines the domain language: `Scan`, `Sweep`, `Radial`, `SweepField`, `Site`. These types are ergonomic, well-documented, and designed for application developers who may not know the NOAA ICD specification. The model crate has no I/O and no parsing — it is a pure data layer.

- **nexrad-decode** implements the NOAA ICD 2620002AA specification. Its types mirror the binary wire format (message headers, data block descriptors, etc.) and are primarily useful for developers working at the protocol level. When the `nexrad-model` feature is enabled, decoded messages can be converted to model types.

This split serves several purposes:

1. **Audience separation** — Application developers work with `nexrad-model` types; protocol developers work with `nexrad-decode` types.
2. **Dependency isolation** — `nexrad-model` has minimal dependencies (optionally `serde`, `chrono`, `uom`). Adding new protocol parsing logic does not affect the model crate's dependency tree.
3. **Stability** — The data model evolves with user-facing requirements. The decoder evolves with the NOAA ICD specification. Decoupling them allows independent versioning.

### Facade Pattern

The `nexrad` crate re-exports all sub-crates and provides top-level convenience functions:

```rust
// Through the facade (ergonomic)
let scan = nexrad::load_file("volume.ar2v")?;
let field = nexrad::extract_field(sweep, Product::Reflectivity);

// Through sub-crates directly (fine-grained)
let file = nexrad_data::volume::File::new(bytes).decompress()?;
let scan = file.scan()?;
```

The facade also provides a unified `nexrad::Error` type that automatically converts from all sub-crate errors via `From` traits.

### Feature Gating

All sub-crate dependencies in the facade are optional and feature-gated:

```toml
# Minimal
nexrad = { version = "1.0", default-features = false, features = ["model", "decode", "data"] }

# WASM-compatible
nexrad = { version = "1.0", default-features = false, features = ["wasm"] }

# Everything (native only)
nexrad = { version = "1.0" }  # default = "full"
```

Features that require a native runtime (`aws-polling` needs tokio, `parallel` needs rayon) are excluded from the `wasm` feature set.

## Data Flow

A typical end-to-end flow through the library:

```
Raw bytes (Archive II file or AWS download)
    │
    ▼  nexrad-data: decompress bzip2 LDM records
Decompressed message bytes
    │
    ▼  nexrad-decode: parse binary protocol
Vec<Message> (low-level ICD types)
    │
    ▼  nexrad-model conversion (via nexrad-data)
Scan { sweeps: [Sweep { radials: [Radial { moments... }] }] }
    │
    ▼  nexrad-model: extract field
SweepField (2D polar grid of f32 + GateStatus)
    │
    ├──▶ nexrad-process: filter / smooth / derive
    │    SweepField, CartesianField, or VerticalField
    │
    └──▶ nexrad-render: color map + rasterize
         RgbaImage (PNG-ready)
```

### Field Types

Processing and rendering operate on three field types, all defined in `nexrad-model`:

| Type | Axes | Use Case |
|------|------|----------|
| `SweepField` | azimuth × range (polar) | Single-sweep PPI display |
| `CartesianField` | lat × lon (geographic) | Composite reflectivity, VIL |
| `VerticalField` | range × altitude | RHI / vertical cross-sections |

## Processing Architecture

Processing algorithms are modeled as traits with three scopes:

| Trait | Scope | Input → Output |
|-------|-------|---------------|
| `SweepProcessor` | Single sweep | `SweepField → SweepField` |
| `ScanProcessor` | Multiple sweeps | `[SweepField] → [SweepField]` |
| `ScanDerivedProduct` | Full scan → geographic surface | `[SweepField] → CartesianField` |

`SweepProcessor` implementations can be composed into a `SweepPipeline`:

```rust
let filtered = SweepPipeline::new()
    .then(ThresholdFilter { min: Some(5.0), max: None })
    .then(GaussianSmooth { sigma_azimuth: 1.0, sigma_range: 1.0 })
    .execute(&field)?;
```

## Rendering Architecture

The renderer converts field data to images through these steps:

1. **Color scale selection** — discrete (step-function) or continuous (linear interpolation)
2. **LUT construction** — `ColorLookupTable` pre-computes 256 RGBA entries for O(1) lookup
3. **Rasterization** — each output pixel is mapped to data coordinates, sampled (nearest-neighbor or bilinear), and colored

Three rendering entry points match the three field types:

| Function | Input | Output |
|----------|-------|--------|
| `render_sweep` | `SweepField` | `RenderResult` (image + metadata) |
| `render_cartesian` | `CartesianField` | `RenderResult` |
| `render_vertical` | `VerticalField` | `RenderResult` |

The legacy `render_radials` function operates directly on `&[Radial]` and returns a plain `RgbaImage` without metadata.

`RenderResult` bundles the image with `RenderMetadata`, enabling pixel-to-geographic coordinate conversion and data value queries — useful for interactive applications and map overlays.

## Versioning and Releases

Each crate is versioned independently. Release tags follow the format `crate-name@x.y.z`. See [releasing.md](releasing.md) for the full release process and dependency ordering.
