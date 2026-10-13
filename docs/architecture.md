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

- **nexrad-decode** implements the NOAA ICD 2620002AA specification. Its types mirror the binary wire format (message headers, data block descriptors, etc.) and are primarily useful for developers working at the protocol level. When the `model` feature is enabled, decoded messages can be converted to model types. All protocol-to-model conversion lives here: `nexrad_decode::model::ScanBuilder` assembles messages into a `Scan`, and `nexrad-data` only decompresses records and feeds their messages to it.

This split serves several purposes:

1. **Audience separation** — Application developers work with `nexrad-model` types; protocol developers work with `nexrad-decode` types.
2. **Dependency isolation** — `nexrad-model` has minimal dependencies (optionally `serde`, `chrono`, `uom`). Adding new protocol parsing logic does not affect the model crate's dependency tree.
3. **Stability** — The data model evolves with user-facing requirements. The decoder evolves with the NOAA ICD specification. Decoupling them allows independent versioning.

### Facade Pattern

The `nexrad` crate re-exports all sub-crates and provides top-level convenience functions:

```rust
// Through the facade (ergonomic)
let scan = nexrad::load_file("volume.ar2v")?;
let field = sweep.field(Product::Reflectivity);

// Through sub-crates directly (fine-grained)
let file = nexrad_data::volume::File::new(bytes).decompress()?;
let scan = file.scan()?;
```

The facade also provides a unified `nexrad::Error` type that automatically converts from all sub-crate errors via `From` traits.

### Feature Gating

All sub-crate dependencies in the facade are optional, depend on the sub-crate with its default features disabled, and
are feature-gated. Each facade feature enables exactly what it names plus the features it builds on: `decode` implies
`model`, `data` implies `decode` (and therefore `model`), and `aws` implies `data`. Network dependencies such as reqwest
are only compiled when `aws` is enabled.

```toml
# Minimal (local files, no networking)
nexrad = { version = "1.0", default-features = false, features = ["data"] }

# WASM-compatible
nexrad = { version = "1.0", default-features = false, features = ["wasm"] }

# Everything (native only)
nexrad = { version = "1.0" }  # default = "full"
```

Features that require a native runtime (`parallel` needs rayon threads) are excluded from the `wasm` feature set. The library does not depend on a specific async runtime: real-time streaming (`chunk_stream` and `nexrad::stream`, part of `aws`) takes a caller-supplied sleep function, so it builds for WASM.

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
    ▼  nexrad-decode: ScanBuilder converts to nexrad-model (driven by nexrad-data)
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
| `CartesianField` | lat × lon (geographic) | Composite reflectivity |
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
    .then(ThresholdFilter::new(Some(5.0), None)?)
    .then(GaussianSmooth::new(1.0, 1.0)?)
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

## Enum Stability

Public enums that are expected to gain variants during 1.x are marked `#[non_exhaustive]`, so adding a variant is not a breaking change. Code outside the defining crate must include a wildcard (`_`) arm when matching on them; code inside the defining crate keeps exhaustive matches so the compiler flags every place a new variant must be handled.

Non-exhaustive (new failure modes, ICD message types, products, and scan strategies are added over time):

- Every crate's `Error` enum (`nexrad-model`, `nexrad-decode`, `nexrad-data`, `nexrad-process`, `nexrad-render`, `nexrad`) and `nexrad_data::result::AwsError`
- `nexrad_decode`: `MessageContents`, `MessageType`, `RedundantChannel`, `DataBlockName`
- `nexrad_model`: `Product`, `VcpNumber`

Exhaustive by design (variants are fixed by the NOAA ICD or by the crate's own format, and several are matched per gate or per radial in hot loops where an exhaustive match is clearer and lets callers rely on the compiler):

- `nexrad_model`: `MomentValue`, `CfpStatus`, `GateStatus`, `RadialStatus`, `PulseWidth`, `WaveformType`, `ChannelConfiguration`
- `nexrad_decode`: `MessageHeaders`, `RadialStatus`, `PulseWidth`, `WaveformType`, `ChannelConfiguration`, `PatternType`, and the small status enums in the message modules
- `nexrad_data`: `ChunkType`, `Chunk`
- `nexrad_render`: `Interpolation`, `ColorScale`

Adding a variant to an exhaustive enum is a breaking change and is reserved for a major release.

## Versioning and Releases

Each crate is versioned independently. Release tags follow the format `crate-name@x.y.z`. See [releasing.md](releasing.md) for the full release process and dependency ordering.
