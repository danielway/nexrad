# nexrad-render

[![Crates.io](https://img.shields.io/crates/v/nexrad-render)](https://crates.io/crates/nexrad-render)
[![Docs.rs](https://docs.rs/nexrad-render/badge.svg)](https://docs.rs/nexrad-render)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/crates/msrv/nexrad-render)](https://github.com/danielway/nexrad#installation)
[![License](https://img.shields.io/crates/l/nexrad-render)](https://github.com/danielway/nexrad/blob/main/LICENSE)

Renders NEXRAD sweeps, composite grids and vertical cross-sections to RGBA images and PNG files.

![Six products from one KDMX volume](https://raw.githubusercontent.com/danielway/nexrad/nexrad-render@1.0.0/docs/images/products.png)

*The lowest tilt of six products from KDMX (Des Moines, Iowa), 5 March 2022 23:23 UTC, each drawn with its `default_scale`. Top row: reflectivity, velocity, spectrum width. Bottom row: differential reflectivity, correlation coefficient, differential phase.*

One of the [nexrad crates](https://github.com/danielway/nexrad). Most programs depend on [`nexrad`](https://crates.io/crates/nexrad), which re-exports this crate.

## Scope

The nexrad crates read NEXRAD Level II data in the Archive II format (ICD 2620010) and the RDA/RPG messages it carries
(ICD 2620002). WSR-88D volumes decode, and so do TDWR Level II volumes. The intended scope for 1.0 is the whole archive:
volumes with legacy message 1 radials, volumes from before 2008 that carry no VCP message, and gzip-compressed volumes
back to 1991 are all inputs. `_MDM` objects (Model Data, message type 29) and archive tarballs are not read, and Level
III products are out of scope.

## Rendering Functions

Three entry points match the three field types from `nexrad-model`:

| Function | Input | Output | Use Case |
|----------|-------|--------|----------|
| `render_sweep` | `SweepField` | `RenderResult` | PPI display (polar → Cartesian) |
| `render_cartesian` | `CartesianField` | `RenderResult` | Composite reflectivity |
| `render_vertical` | `VerticalField` | `RenderResult` | RHI / vertical cross-sections |

To render a sweep of radials, build a `SweepField` first with `sweep.field(product)` or
`SweepField::from_radials(radials, product)`:

```rust
use nexrad_model::data::{Product, Sweep};
use nexrad_render::{default_scale, render_sweep, RenderOptions};

fn render(sweep: &Sweep) -> nexrad_render::Result<()> {
    let field = sweep.field(Product::Reflectivity).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let result = render_sweep(&field, &scale, &RenderOptions::new(800, 800))?;
    result.save_png("reflectivity.png")?;
    Ok(())
}
```

## RenderOptions

Configure rendering via the builder API. Dimensions are `u32`; rendering with a zero
width or height returns `Error::InvalidDimensions`.

```rust
use nexrad_model::data::{GateStatus, SweepField};
use nexrad_model::geo::{GeoExtent, RadarCoordinateSystem};
use nexrad_render::{Color, Interpolation, RenderOptions};

fn options(field: &SweepField, extent: GeoExtent, coord_system: RadarCoordinateSystem) {
    // Basic: fixed size with black background
    let options = RenderOptions::new(800, 800);

    // Transparent background (for compositing)
    let options = RenderOptions::new(800, 800).with_background(Color::TRANSPARENT);

    // Native resolution (one pixel per gate at the outer edge)
    let options = RenderOptions::native_for(field);

    // Bilinear interpolation (smoother output)
    let options = RenderOptions::new(800, 800).with_interpolation(Interpolation::Bilinear);

    // Fixed color range and purple range-folded gates
    let options = RenderOptions::new(800, 800)
        .with_value_range(-32.0, 95.0)
        .with_status_color(GateStatus::RangeFolded, Color::rgb(0.5, 0.0, 0.5));

    // Render a sweep onto a lat/lon grid covering a fixed geographic extent
    let options = RenderOptions::new(800, 800)
        .with_extent(extent)
        .with_coord_system(coord_system);
}
```

Without `with_value_range`, the color lookup range is the field's product range, else the
range of its valid values, else `-32.0..95.0`. Gates with a non-`Valid` status are drawn as the background unless
`with_status_color` sets a color for that status. `render_cartesian` and `render_vertical`
ignore the extent: their fields carry their own.

## Color Scales

Built-in color scales (each function's documentation lists its colors):

| Function | Product | Range |
|----------|---------|-------|
| `reflectivity_scale()` | Reflectivity | -32 to 95 dBZ |
| `velocity_scale()` | Radial Velocity | -64 to +64 m/s |
| `spectrum_width_scale()` | Spectrum Width | 0 to 30 m/s |
| `differential_reflectivity_scale()` | ZDR | -2 to +6 dB |
| `correlation_coefficient_scale()` | CC/RhoHV | 0 to 1 |
| `differential_phase_scale()` | PhiDP | 0 to 360° |
| `clutter_filter_power_scale()` | CFP | -20 to +20 dB |

Use `default_scale(product)` to select the appropriate scale as a `ColorScale`; the
individual functions above return a `DiscreteColorScale`, which converts with `.into()`.

Custom scales can be built with `DiscreteColorScale` (step-function) or `ContinuousColorScale` (linear interpolation).
`ColorLookupTable::new(&scale, min, max, size)` pre-computes a scale for O(1) lookups; it returns
`Error::InvalidRange` unless `min < max` (both finite) and `Error::InvalidTableSize` for fewer than 2
entries. `entries()` exposes the RGBA table (for example as a 1-D GPU texture), and
`DiscreteColorScale::levels()` exposes each level's public `value` and `color` for legends.

## RenderResult and Metadata

`render_sweep`, `render_cartesian`, and `render_vertical` return a `RenderResult` that bundles
the rendered pixels with spatial metadata:

```rust
use nexrad_model::data::SweepField;
use nexrad_model::geo::GeoPoint;
use nexrad_render::{render_sweep, ColorScale, RenderOptions};

fn render(
    field: &SweepField,
    color_scale: &ColorScale,
    options: &RenderOptions,
    geo_point: GeoPoint,
) -> nexrad_render::Result<()> {
    let result = render_sweep(field, color_scale, options)?;

    // Raw RGBA8 pixels, row-major (for a GPU texture or another image library)
    let (width, height) = (result.width(), result.height());
    let rgba: &[u8] = result.pixels();

    // Save as PNG, or encode PNG into memory
    result.save_png("radar.png")?;
    let mut png = Vec::new();
    result.write_png(&mut png)?;

    // Query data values at specific coordinates (pass the field that was rendered)
    let meta = result.metadata();
    let query = meta.query_pixel(field, 400.0, 300.0);
    let query = meta.query_geo(field, &geo_point);

    // Use metadata for map overlays
    let ring_px = meta.km_to_pixel_distance(100.0);  // Some(..): 100km range ring
    let pixel = meta.polar_to_pixel(180.0, 50.0);    // Some((x, y)): azimuth 180°, range 50km
    Ok(())
}
```

`RenderMetadata::projection()` reports how pixels map to data coordinates:

| Projection | Produced by | Supported conversions |
|------------|-------------|-----------------------|
| `Projection::Polar` | `render_sweep` | polar ↔ pixel, km ↔ pixel distance, geo ↔ pixel (with a coordinate system), `query_pixel`/`query_polar`/`query_geo` |
| `Projection::Geographic` | `render_sweep` with an extent, `render_cartesian` | geo ↔ pixel (linear through the extent), `query_pixel`/`query_polar`/`query_geo` for sweeps, `query_cartesian` |
| `Projection::Vertical` | `render_vertical` | `pixel_to_vertical`/`vertical_to_pixel`, `query_vertical` |

Conversions a projection does not support return `None`.

## Pixels, PNG and the `image` feature

A `RenderResult` owns its pixels as 8-bit RGBA, row-major with no row padding: `pixels()` borrows
them and `into_pixels()` hands over the `Vec<u8>`. No image-library type appears in the API.
`write_png` and `save_png` encode PNG with a private encoder; encoding failures surface as
`Error::Encode` (with the encoder's error as its source) and write failures as `Error::Io`. For
other formats, pass the pixels to the image library of your choice.

The opt-in `image` feature adds `RenderResult::into_rgba_image()`, which converts a result into an
`image::RgbaImage`:

```toml
nexrad-render = { version = "1.0", features = ["image"] }
```

This feature tracks the `image` crate's own 0.x releases and is excluded from the semver promise:
a minor or patch release of `nexrad-render` may move it to a newer, incompatible `image` version.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.
To upgrade from 0.x or a release candidate, see [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).

## Crate Boundaries

This crate provides **visualization and rendering** only:

- No data access or network operations
- No binary parsing or decoding

For most users, the [`nexrad`](https://crates.io/crates/nexrad) facade crate is the recommended entry point.
