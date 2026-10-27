# Architecture

This document describes the design philosophy and structure of the NEXRAD Rust library suite, and the rules the six
library crates follow.

## Mono-Repo Structure

The project is a Cargo workspace with six published library crates, each with a single responsibility, and two
unpublished tools. Users can depend on individual crates or on the `nexrad` facade.

```
                  nexrad-model        Pure data types (no I/O, no parsing)
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
 nexrad-decode   nexrad-process   nexrad-render
 Binary protocol Processing       Visualization
 parsing + model algorithms       (data in, images out)
 conversion            │                │
       │               │                │
       ▼               │                │
  nexrad-data          │                │
  I/O boundary: files, │                │
  decompression, AWS   │                │
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                    nexrad            Facade: re-exports + I/O entry points

 nexrad-inspector   Interactive TUI (binary, publish = false)
 xtask              Repository maintenance tasks (publish = false)
```

Arrows point from a crate to the crates that depend on it.

## Design Principles

### Separation of Concerns

Each library crate owns exactly one concern:

| Crate | Owns | Does NOT do |
|-------|------|-------------|
| `nexrad-model` | Domain types and the methods on them, site registry | I/O, parsing, rendering |
| `nexrad-decode` | Binary protocol parsing per NOAA ICD, all protocol-to-model conversion (`model` feature) | File I/O, network access, decompression |
| `nexrad-data` | Archive II file handling, decompression, AWS S3, real-time chunk polling | Protocol-to-model mapping, rendering, processing |
| `nexrad-process` | Filtering, smoothing, derived products, threshold region detection | I/O, rendering |
| `nexrad-render` | Color mapping, image generation, pixel ↔ data coordinate mapping | Data access, parsing |
| `nexrad` | Re-exports, I/O entry points, unified error type | New functionality |

A WASM application can therefore use `nexrad-model` and `nexrad-render` without file I/O or native networking code.
`nexrad-render` returns rendered images in memory as owned RGBA8 pixels; its only I/O is `write_png`/`save_png`,
which encode a rendered image as PNG to a caller-supplied writer or file path.

### Layering Rules

- **All conversion to model types lives in `nexrad-decode`**, behind its `model` feature (for example
  `nexrad_decode::model::ScanBuilder`).
- **`nexrad-data` orchestrates I/O and decompression only.** `volume::File::scan` (which also decompresses a gzip-wrapped
  file), `volume::Record::radials` and `aws::realtime::assemble_volume` decompress records and hand their messages to
  `nexrad-decode`, with no protocol-to-model mapping of their own.
- **The facade contains re-exports and I/O entry points only** (`load`, `download`, `list_scans`, `stream` and their
  variants). Anything expressible as a method on a model type lives on the model type (`Sweep::field`), and the static
  site registry lives in `nexrad_model::meta::registry`.

### Why nexrad-model and nexrad-decode Are Separate

- **nexrad-model** defines the domain language (`Scan`, `Sweep`, `Radial`, `SweepField`, `Site`) for application
  developers who may not know the NOAA ICD. It has no I/O and no parsing.
- **nexrad-decode** implements NOAA ICD 2620002AA. Its types mirror the binary wire format for developers working at the
  protocol level, and its `model` feature converts them to model types.

The split keeps the model's dependencies minimal (`thiserror`, and optionally `serde`, `chrono`) and untouched by
new protocol parsing, and keeps wire-format details out of the model's public API: the model evolves with user-facing
requirements, the decoder with the ICD.

Five types exist in both crates: `WaveformType`, `ChannelConfiguration`, `PulseWidth`, `RadialStatus` and `VcpNumber`.
The two copies of each have the same variants, names, payloads, `Display` and `serde` form, and the `From` impls between
them (decode's `model` feature) convert every value in both directions. Only the decode copy knows the ICD codes
(`from_number`/`number`); the model's `Unknown(u8)` carries the code the decoder read without interpreting it.

### Facade Pattern

The `nexrad` crate re-exports every sub-crate and provides the top-level I/O entry points and a unified `nexrad::Error`
that converts from every sub-crate error via `From` (see [Error Architecture](#error-architecture)):

```rust
// Through the facade (ergonomic)
let scan = nexrad::load_file("volume.ar2v")?;
let field = scan.sweeps()[0].field(Product::Reflectivity);

// Through sub-crates directly (fine-grained)
let scan = nexrad_data::volume::File::new(bytes).scan()?;
```

## Public API and Versioning Policy

### What the public API is

Everything reachable through `pub` is public API and is covered by semver. The library crates have no `#[doc(hidden)]`
items: the examples and `nexrad-inspector` use only the documented public API, and an item one of them needs is made
public and documented, not hidden. The only items outside the public API are those compiled for doc tests alone
(`#[cfg(doctest)]`, which wires each crate's README into `cargo test --doc`); they do not exist in a normal build.

Each library crate sets `[package.metadata.docs.rs] all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]` so
docs.rs shows the features feature-gated items require, enables `#![warn(unnameable_types)]` so a public type cannot
leak through a private module path, and enables `#![warn(missing_debug_implementations)]` so every public type
implements `Debug`.

### Versioning

Each crate is versioned independently, keeps a `CHANGELOG.md` in the
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format, and is released bottom-up (see
[releasing.md](releasing.md#release-order)). `nexrad-inspector` and `xtask` are `publish = false`.

### Dependency Exposure

An external type in a public signature ties our major version to that dependency's. Only these external crates may
appear in public signatures:

| Crate | Where | Notes |
|-------|-------|-------|
| `std` | Everywhere | |
| `chrono` | Timestamps and dates in model, decode, data and the facade | Re-exported from the facade as `nexrad::chrono` (feature `chrono`) |
| `serde` | `Serialize`/`Deserialize` impls | Opt-in `serde` feature |
| `image` | `nexrad_render::RenderResult::into_rgba_image` only | Opt-in `image` feature of `nexrad-render` (and the facade), off by default and not in `full`. It tracks `image`'s own 0.x releases and is excluded from the semver promise. Without it, `RenderResult` exposes owned RGBA8 pixels (`pixels()`, `into_pixels()`) |
| `futures_core` | The `Stream` returned by `nexrad_data::aws::realtime::chunk_stream` and `nexrad::stream` | Return type only |
| `futures_util` | The facade, internally | Not re-exported and not in any signature; consumers depend on `futures-util` or `futures` themselves for `StreamExt` |

`chrono` and `futures_core` are 0.x crates. The 1.x releases stay on `chrono` 0.4 and `futures-core` 0.3; moving to a
later major version of either is a major release of every crate that exposes it.

Not allowed in public signatures: `reqwest`, `bzip2`, `png`, `zerocopy` traits, `tokio`, `once_cell`. HTTP failures are
boxed inside `AwsError`, bzip2 failures surface as `Error::Io`, PNG encoder failures are boxed inside
`nexrad_render::Error::Encode`, and the `#[repr(C)]` wire structs of `nexrad-decode` and
`nexrad-data` stay private.

### Minimum Supported Rust Version

The MSRV is declared once, as `rust-version = "1.88.0"` in `[workspace.package]`, and verified by the `MSRV check` CI
job. It is raised only in minor releases, never in patch releases, and every raise is noted in the affected crates'
CHANGELOGs.

### Enforcement Tooling

| CI job | What it checks |
|--------|----------------|
| `MSRV check` | `cargo check --workspace --all-features --all-targets` and `cargo hack check --workspace --each-feature --optional-deps` on Rust 1.88.0 |
| `Public API baselines` | `api/<crate>.txt` matches each library crate's public API. Regenerate with `cargo run -p xtask -- public-api`; compare with `cargo run -p xtask -- public-api --check`. A PR that changes a public API updates the baselines in the same PR |
| `Semver checks` | `cargo semver-checks` against the latest release on crates.io. Advisory (`continue-on-error: true`) until 1.0.0 is published, then blocking |
| `Fuzz target build check` | The fuzz targets in `nexrad-decode/fuzz` and `nexrad-data/fuzz` (outside the workspace) still build |
| `WASM compatibility check` | Each library crate, and the facade with every feature except `parallel` and `image`, builds for `wasm32-unknown-unknown` |
| `Feature set tests` | `cargo test -p nexrad` with no default features and with every feature except `parallel` and `image` |
| `Windows build` | `cargo check --workspace --all-features --all-targets` on Windows, which builds the `aws-rustls` TLS stack (aws-lc-rs with NASM) |

## Feature Architecture

Every crate uses `dep:` syntax for optional dependencies, so an optional dependency's name is never a feature. Features
are named for what they enable, not for the crate they pull in.

| Crate | Features (defaults in bold) |
|-------|-----------------------------|
| `nexrad-model` | `serde`, `chrono` |
| `nexrad-decode` | **`model`** (conversion to `nexrad-model` types), `serde` (the code enums and `VcpNumber`) |
| `nexrad-data` | **`aws`** (S3 archive and real-time access, without a TLS backend), **`aws-rustls`** (`aws` with rustls and the aws-lc-rs provider), **`model`** (`File::scan`, `Record::radials`, `assemble_volume`; enables `nexrad-decode/model`), `parallel` (rayon) |
| `nexrad-process` | none |
| `nexrad-render` | `image` (`RenderResult::into_rgba_image`; outside the semver promise) |
| `nexrad` | **`data`**, **`aws`**, **`aws-rustls`**, **`render`**, `full` for every feature except `image`, and `image`; see the feature table in the crate docs |

No crate exposes unit-typed quantities: accessors return plain numbers with the unit in the name. `uom` was removed
before 1.0 because it is pre-1.0 itself and would tie our major version to its own; unit-typed accessors may come later
as a separate crate or extension trait.

### Feature Policy

- **Defaults are a working set, not everything.** The facade's default is "load, download, render" (`data`, `aws`,
  `aws-rustls`, `render`). `full` enables every feature except `image`. Removing a feature, or removing one from a
  default set, is a breaking change, while adding one is not, so the defaults stay small.
- **No target-named feature sets.** Every feature except `parallel` builds for `wasm32-unknown-unknown`, and a WASM
  build lists the features it uses. CI checks that set for `wasm32` and tests it on native.
- **Interop outside the semver promise stays opt-in.** `image` (`RenderResult::into_rgba_image`) tracks the `image`
  crate's own 0.x releases and may move to a newer, incompatible `image` version in a minor or patch release, so it
  is in neither the default set nor `full`.
- **TLS backends are separate features.** `aws` is the HTTP client machinery with no TLS backend; on `wasm32` it uses
  the JavaScript `fetch` API and needs none. `aws-rustls` (default) selects rustls with the aws-lc-rs provider, which
  needs a C compiler and, on Windows x86 targets, NASM. Other backends can be added later as further `aws-*` features
  (for example `aws-rustls-ring` or `aws-native-tls`) without breaking anyone. On native targets, `aws` without any
  TLS backend compiles, but every HTTPS request fails at run time with reqwest's `invalid URL, scheme is not http`
  connect error; a TLS feature of `reqwest` (for example `native-tls`) enabled by the user's manifest or any other
  crate in the build satisfies it, since Cargo unifies features.

### Facade Feature Gating

All sub-crate dependencies in the facade are optional, feature-gated and depend on the sub-crate with its default
features disabled. Each facade feature enables exactly what it names plus the features it builds on (the crate docs
list them), so network dependencies such as reqwest are compiled only with `aws`.

### No Async Runtime Dependency

No library crate depends on an async runtime, and `aws` is the only network feature. Real-time streaming
(`chunk_stream(config, sleep)`, `nexrad::stream(site, sleep)`) takes a caller-supplied sleep function, such as
`|d| tokio::time::sleep(d)` or a `gloo-timers` timer on `wasm32`, so it builds for WASM. Only `parallel` (rayon threads)
does not build for WASM.

## Error Architecture

- Every crate defines its errors in a public `result` module and re-exports `Error` and `Result` at its root;
  `nexrad-data` also re-exports `AwsError` (feature `aws`).
- Every `Error` enum, and `nexrad_data::result::AwsError`, is `#[non_exhaustive]`, and so is each of their struct-like
  variants, so a variant can gain a field in a minor release. Code outside the defining crate builds these errors only
  through the operations that return them.
- `nexrad_process::Error::Custom` boxes the error of a `SweepProcessor` implemented outside the crate.
- No error carries a 0.x dependency's error type as a payload. HTTP errors (`nexrad_data::AwsError`) and image
  encoder errors (`nexrad_render::Error::Encode`) are boxed as `Box<dyn std::error::Error + Send + Sync>`, which
  `source()` returns; I/O failures while writing an image are `nexrad_render::Error::Io(std::io::Error)`.
- `nexrad_data::Error::Decode`, `Model` and `Io` wrap the decode, model and I/O errors raised while reading data.
- The facade's `From<nexrad_data::Error>` flattens those three into `nexrad::Error::Decode`, `Model` and `Io`, so each
  facade variant names the layer an error came from; every other data error (decompression, record layout, AWS)
  becomes `nexrad::Error::Data`. `nexrad::Error::NoDataAvailable` and `NoScanAt` exist only with the `aws` feature.

## Serialization

With the `serde` feature, `nexrad-model` types serialize with field names equal to their accessor names, enum variants
as their PascalCase names, `VcpNumber` as its number and raw gate bytes as a byte string. `nexrad-decode`'s `serde`
feature covers only the five types it shares with the model, in the same form. Deserialization applies the
same validation as the constructors and builders. The representation is stable within a major version: a field added
to a serialized type in a minor release carries `#[serde(default)]`, so data written by an earlier 1.x release still
deserializes.

## Naming Conventions

- **Acronyms are UpperCamelCase words**: `VcpNumber`, `AwsError`.
- **Decoded-message accessors** in `nexrad-decode` follow "Accessor Naming" in its crate docs: the stem is the ICD
  field name; `<stem>_raw()` exists only when the wire encoding differs from the decoded value; a decoded plain number
  is `<stem>_<unit>()` (`azimuth_angle_degrees()`); and a decoded enum or struct is the bare stem
  (`radial_status()`).
- **Model types** keep unit suffixes on plain numbers. Angles use the stems `azimuth_angle_degrees` and
  `elevation_angle_degrees` (a slice of azimuths is `azimuth_angles_degrees`); lengths end in `_meters` or `_km`,
  velocities in `_mps`, and timestamps in `_millis`. `site_elevation_meters` is the ground elevation above mean sea
  level (MSL) and `feedhorn_height_meters` the feedhorn's height above ground level (AGL).
- **Numeric types**: geographic coordinates and ranges are `f64`; gate values (and the angles stored alongside them on
  `Radial`, `Sweep` and `SweepField`) are `f32`; image dimensions are `u32`.
- **Constructors**: types with invariants have private fields, and their constructors and builders validate the
  invariants and return a `Result` (`RadialBuilder::build`, `Sweep::new`); field indexing returns `Option`, and `set`
  returns a `Result`. A builder whose type could gain an invariant returns a `Result` even when it checks little today
  (`VolumeCoveragePatternBuilder::build`), because changing `T` to `Result<T>` later is breaking and adding an error
  variant is not. Constructors with more than a few arguments, or with arguments of the same type that could be
  swapped, are builders (`MomentData::builder`). Configuration types (`RetryPolicy`, `PollConfig`) have constructors with `with_*` setters and
  getters. Every `nexrad-process` processor with parameters has a validating `new` or builder.
- **Lifetimes**: no public function returns a value with an unbounded output lifetime (`Record::decompress` returns
  `Record<'static>`).

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
    ▼  nexrad-model: Sweep::field / Scan::fields
SweepField (2D polar grid of f32 + GateStatus)
    │
    ├──▶ nexrad-process: filter / smooth / derive
    │    SweepField, CartesianField, or VerticalField
    │
    └──▶ nexrad-render: color map + rasterize
         RenderResult (RGBA8 pixels + RenderMetadata)
```

### Moment Data

Every moment on a `Radial`, including clutter filter power, is a `MomentData` that carries its `Product`, and the
radial carries its own Nyquist velocity and unambiguous range when the source provides them. `Radial::moment(product)`
is the uniform accessor (`reflectivity()` and the other named getters are shorthands), gates decode to `MomentValue`,
and `SweepField::product()` records which product a field was extracted from.

### Field Types

Processing and rendering operate on three field types, all defined in `nexrad-model`:

| Type | Axes | Use Case |
|------|------|----------|
| `SweepField` | azimuth × range (polar) | Single-sweep PPI display |
| `CartesianField` | lat × lon (geographic) | Composite reflectivity |
| `VerticalField` | range × altitude | RHI / vertical cross-sections |

## Processing Architecture

Single-sweep algorithms implement the `SweepProcessor` trait (`SweepField → SweepField`); filters such as
`ThresholdFilter` are `SweepProcessor`s. Products computed from the sweeps of a full scan have inherent `compute` methods
rather than a shared trait: `CompositeReflectivity` returns a `CartesianField` on a `CartesianGrid` and
`VerticalCrossSection` a `VerticalField`. A trait for scan products can be added in a minor release once more than one
product shows its shape. `ThresholdRegionDetector` also stands alone. Processors with parameters are built with a validating
`new` or builder that returns `Err(Error::InvalidParameter)`, so invalid configuration is reported at construction
rather than by `process`.

`SweepProcessor` implementations compose into a `SweepPipeline`:

```rust
let filtered = SweepPipeline::new()
    .then(ThresholdFilter::new(Some(5.0), None)?)
    .then(GaussianSmooth::new(1.0, 1.0)?)
    .execute(&field)?;
```

## Rendering Architecture

The renderer converts field data to images in three steps:

1. **Color scale selection**: a `ColorScale` is discrete (step-function) or continuous (linear interpolation);
   `default_scale(product)` picks the crate's built-in scale for a product.
2. **LUT construction**: `ColorLookupTable::new(&scale, min, max, size)` pre-computes RGBA entries for O(1) lookup;
   renderers build it over `RenderOptions::value_range()`, else the field's product range, else the field's valid-value
   range.
3. **Rasterization**: each output pixel is mapped to data coordinates, sampled (nearest-neighbor or bilinear), and
   colored; gates with a non-`Valid` status take `RenderOptions::status_color(status)` or stay background.

Three rendering entry points match the three field types:

| Function | Input | Output | Projection |
|----------|-------|--------|------------|
| `render_sweep` | `SweepField` | `RenderResult` (pixels + metadata) | `Projection::Polar`; `Projection::Geographic` with `RenderOptions::with_extent` (each pixel mapped through the coordinate system to the sweep) |
| `render_cartesian` | `CartesianField` | `RenderResult` | `Projection::Geographic` |
| `render_vertical` | `VerticalField` | `RenderResult` | `Projection::Vertical` |

`RenderResult` bundles an owned RGBA8 pixel buffer (`pixels()`, `into_pixels()`, `write_png`, `save_png`) with
`RenderMetadata`, whose `projection()` describes how pixels map back to data coordinates. Conversions a projection
supports return `Some` and the rest `None`. Data value queries live on `RenderMetadata` and take the field that was
rendered (`query_pixel` for sweeps, `query_cartesian` for grids, and so on).

## Realtime Subsystem

`nexrad_data::aws::realtime` (feature `aws`) polls the `unidata-nexrad-level2-chunks` bucket for live volume chunks.
`ChunkIterator` (pull-based, with timing estimates such as `next_expected_time`) and `chunk_stream(config, sleep)`
(a push-based `Stream`) are configured with `PollConfig` and `RetryPolicy`; one-shot calls such as `download_chunk` and
`get_latest_volume` sit beside them, and `assemble_volume` (feature `model`) feeds chunk messages to `nexrad-decode`'s
`ScanBuilder`.

Both `ChunkIterator` and `chunk_stream` consume a new volume from its Start chunk (sequence 1, carrying the volume
header and VCP) and then in sequence order, even when later chunks were published while they waited.

The chunk timing model, the elevation-to-chunk mapper, retry state and scan timing projection are crate-private; their
results are exposed only through the `ChunkIterator` estimates.

## Enum Stability

Public enums that are expected to gain variants in minor releases are marked `#[non_exhaustive]`, so adding a variant
is not a breaking change. Code outside the defining crate must include a wildcard (`_`) arm when matching on them; code
inside the defining crate keeps exhaustive matches so the compiler flags every place a new variant must be handled.

Non-exhaustive: every crate's `Error` enum and `nexrad_data::result::AwsError`, and enums over sets that grow with the
ICD or the library, such as message types and products (`MessageContents`, `Product`) and render projections
(`Projection`). `nexrad_model::data::GateStatus` is non-exhaustive because some of its variants (`Filtered`, `NoData`)
are set by processing steps rather than by the data encoding, and a later processing step may add a status.

`VcpNumber` is not an enum: it is a `u16` newtype with associated constants (`VcpNumber::VCP_212`) that work as `match`
patterns. Naming a further pattern adds a constant, which cannot change how an existing value matches, compares or
serializes.

Non-exhaustive structs and enum variants (fields may be added in minor releases, so code outside the defining crate
cannot construct them with a literal and must use `..` in struct patterns): `nexrad_model::meta::registry::SiteEntry`,
`nexrad_render::PointQuery`, the `Projection::Polar`, `Geographic` and `Vertical` variants, and every struct-like
variant of every crate's `Error` and of `nexrad_data::AwsError` (such as `AwsError::S3GetObject { status, body }`).

`nexrad_data::aws::realtime::ChunkType` is non-exhaustive as well: the Archive II ICD defines a fourth LDM key type,
`M` (Model Data), that the real-time bucket does not use today.

The ICD code enums in the `nexrad-decode` message modules that carry an `Unknown(n)` or `Reserved(n)` variant (the
status, mode and code enums of the RDA status, clutter, radar data and VCP messages) are also non-exhaustive, and so are
their `nexrad-model` counterparts (`WaveformType`, `ChannelConfiguration`, `PulseWidth`, `RadialStatus`) and
`nexrad_model::data::CfpStatus`, as are `ColorScale` and `Interpolation` in `nexrad-render` and the unit struct
`nexrad_process::derived::CompositeReflectivity`. A variant that a newer `nexrad-model` names converts into an older
`nexrad-decode` as `Unknown(255)`.
An `Unknown(n)` or `Reserved(n)` value may be promoted to a named variant in a minor release when a later ICD names that
code; code that needs a stable result across releases should compare the raw code (`number()` or the `*_raw()`
accessor) rather than match on `Unknown(n)`.

Exhaustive: enums whose variants are fixed by the NOAA ICD or by the crate's own format, such as `MomentValue` (a
value, below threshold, range folded or a clutter filter power status code, as the ICD encodes a gate). Adding a
variant to an exhaustive enum is a breaking change and is reserved for a major release.

## Versioning and Releases

Release tags follow the format `crate-name@x.y.z`. See [releasing.md](releasing.md) for the release process.
