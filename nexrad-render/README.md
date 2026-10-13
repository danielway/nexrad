# NEXRAD Render

[![Crates.io](https://img.shields.io/crates/v/nexrad-render)](https://crates.io/crates/nexrad-render)
[![Docs.rs](https://docs.rs/nexrad-render/badge.svg)](https://docs.rs/nexrad-render)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Functions for rendering NEXRAD weather radar data into visual images. Supports multiple radar
products including reflectivity, velocity, and spectrum width with configurable color scales.

## Rendering Functions

Three entry points match the three field types from `nexrad-model`:

| Function | Input | Output | Use Case |
|----------|-------|--------|----------|
| `render_sweep` | `SweepField` | `RenderResult` | PPI display (polar → Cartesian) |
| `render_cartesian` | `CartesianField` | `RenderResult` | Composite reflectivity |
| `render_vertical` | `VerticalField` | `RenderResult` | RHI / vertical cross-sections |

To render a sweep of radials, build a `SweepField` first with `sweep.field(product)` or
`SweepField::from_radials(radials, product)`:

```rust,ignore
use nexrad_render::{default_scale, render_sweep, Product, RenderOptions};

let field = sweep.field(Product::Reflectivity).unwrap();
let scale = default_scale(Product::Reflectivity);
let result = render_sweep(&field, &scale, &RenderOptions::new(800, 800))?;
result.save("reflectivity.png")?;
```

## RenderOptions

Configure rendering via the builder API. Dimensions are `u32`; rendering with a zero
width or height returns `Error::InvalidDimensions`.

```rust,ignore
use nexrad_render::{RenderOptions, Interpolation};

// Basic: fixed size with black background
let options = RenderOptions::new(800, 800);

// Transparent background (for compositing)
let options = RenderOptions::new(800, 800).transparent();

// Native resolution (one pixel per gate at the outer edge)
let options = RenderOptions::native_for(&field);

// Bilinear interpolation (smoother output)
let options = RenderOptions::new(800, 800).bilinear();

// Geographic extent for consistent spatial mapping
let options = RenderOptions::new(800, 800)
    .with_extent(extent)
    .with_coord_system(coord_system);
```

## Color Scales

Built-in color scales follow standard meteorological conventions:

| Function | Product | Range |
|----------|---------|-------|
| `nws_reflectivity_scale()` | Reflectivity | -32 to 95 dBZ |
| `velocity_scale()` | Radial Velocity | -64 to +64 m/s |
| `spectrum_width_scale()` | Spectrum Width | 0 to 30 m/s |
| `differential_reflectivity_scale()` | ZDR | -2 to +6 dB |
| `correlation_coefficient_scale()` | CC/RhoHV | 0 to 1 |
| `differential_phase_scale()` | PhiDP | 0 to 360° |
| `clutter_filter_power_scale()` | CFP | -20 to +20 dB |

Use `default_scale(product)` to select the appropriate scale as a `ColorScale`; the
individual functions above return a `DiscreteColorScale`, which converts with `.into()`.

Custom scales can be built with `DiscreteColorScale` (step-function) or `ContinuousColorScale` (linear interpolation).
`ColorLookupTable::new(&scale, min, max, size)` pre-computes a scale for O(1) lookups.

## RenderResult and Metadata

`render_sweep`, `render_cartesian`, and `render_vertical` return a `RenderResult` that bundles the image with spatial metadata:

```rust,ignore
let result = render_sweep(&field, &color_scale, &options)?;

// Save the image
result.save("radar.png")?;

// Query data values at specific coordinates (polar renders only)
let query = result.query_pixel(&field, 400.0, 300.0);
let query = result.query_geo(&field, &geo_point);

// Use metadata for map overlays
let meta = result.metadata();
let ring_px = meta.km_to_pixel_distance(100.0);  // Some(..): 100km range ring
let pixel = meta.polar_to_pixel(180.0, 50.0);    // Some((x, y)): azimuth 180°, range 50km
```

`RenderMetadata::projection()` reports how pixels map to data coordinates:

| Projection | Produced by | Supported conversions |
|------------|-------------|-----------------------|
| `Projection::Polar` | `render_sweep` | polar ↔ pixel, km ↔ pixel distance, geo ↔ pixel (with a coordinate system), point queries |
| `Projection::Geographic` | `render_cartesian` | geo ↔ pixel (linear through the extent) |
| `Projection::Vertical` | `render_vertical` | none (axis ranges are in the projection) |

Conversions a projection does not support return `None`.

## The `image` dependency

Images are `image::RgbaImage` values and I/O failures surface as `Error::Image(image::ImageError)`,
so the `image` crate is part of the public API and is re-exported as `nexrad_render::image`.
A major version bump of `image` is a major version bump of `nexrad-render`.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.

## Crate Boundaries

This crate provides **visualization and rendering** only:

- No data access or network operations
- No binary parsing or decoding

For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
