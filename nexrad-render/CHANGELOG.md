# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.4`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root (`nexrad_render::Error`); `nexrad_render::result` remains
  the defining module. (#158)
- `Projection`, a `#[non_exhaustive]` enum describing how a rendered image maps to data coordinates:
  `Polar { center_pixel, pixels_per_km, max_range_km, elevation_angle_degrees }`, `Geographic { extent }` and
  `Vertical { distance_range_km, altitude_range_meters }`; read it with `RenderMetadata::projection()`. Each variant
  is also `#[non_exhaustive]`, so struct patterns outside this crate need `..`. (#174, #182)
- `pub use image;`: the `image` crate is re-exported at the crate root, alongside `RgbaImage`. An `image` major
  version bump is a `nexrad-render` major version bump. (#174)
- `ColorLookupTable::new(&ColorScale, min, max, size)`. (#174)
- `Error::EmptyField` and `Error::Image(image::ImageError)`. (#174)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- Polar angles are `f64`: `RenderMetadata::elevation_degrees()` returns `Option<f64>`,
  `RenderMetadata::polar_to_pixel` and `RenderResult::query_polar` take `azimuth_degrees: f64`, and
  `PointQuery::polar` follows the `f64` `PolarPoint`. (#164)
- `default_scale(Product)` returns a `ColorScale` instead of a `DiscreteColorScale`. (#174)
- Image dimensions are `u32`: `RenderOptions::new(u32, u32)`, `RenderOptions::size() -> (u32, u32)`, and
  `RenderOptions::native_for` computes a `u32` size (saturating at `u32::MAX`). (#174)
- `RenderResult::save` fails with `Error::Image(image::ImageError)` instead of `ImageSave(String)`, preserving the
  source error. (#174)
- `Error` no longer implements `UnwindSafe`/`RefUnwindSafe`, because `image::ImageError` does not; wrap it in
  `std::panic::AssertUnwindSafe` where that matters. (#174)
- `Error::NoRadials` → `Error::EmptyField`, returned for a sweep field with no radials and for a Cartesian or vertical
  field with no rows or columns. `Error::InvalidDimensions` is also returned for a zero width or height or a buffer
  size that overflows `usize`. (#174)
- `RenderMetadata` accessors that only make sense for polar renders return `Option`: `center_pixel()`,
  `pixels_per_km()`, `max_range_km()`, `polar_to_pixel()`, `pixel_distance_to_km()` and `km_to_pixel_distance()` are
  `Some` only for `Projection::Polar`, as are `elevation_degrees()` (now `elevation_angle_degrees()`) and
  `pixel_to_polar()`. `RenderResult::query_pixel`, `query_polar` and `query_geo` return `None` unless the projection is
  `Polar`. (#174)
- `RenderMetadata::pixel_to_geo` and `geo_to_pixel` map linearly through the extent for `render_cartesian` results
  and return `None` for `render_vertical` results; polar behaviour is unchanged. (#174)
- `RenderResult::new` is crate-private; it could not be called usefully outside the crate. (#174)
- `RenderMetadata::elevation_degrees()` → `elevation_angle_degrees()`, matching the `nexrad-model` angle stems. (#182)
- `PointQuery` is `#[non_exhaustive]`; it can no longer be built with a struct literal outside this crate. Its `polar`
  field follows the renamed `PolarPoint::{azimuth_angle_degrees, elevation_angle_degrees}`. (#182)
- When a field has no valid data, `render_sweep` picks its color lookup range from the field's product
  (`Product::value_range`) before the reflectivity default. Rendered output does not change. (#174)

### Removed

- `render_radials` and `render_radials_default`. Build a field with `sweep.field(product)` or
  `SweepField::from_radials(radials, product)`, then call `render_sweep(&field, &scale, &options)`. (#174)
- `default_color_scale(Product)`; use `default_scale(Product)`. (#174)
- `ColorLookupTable::from_scale` and `ColorLookupTable::from_color_scale`; use `ColorLookupTable::new`, converting a
  `DiscreteColorScale` with `.into()`. (#174)
- `Error::ProductNotFound` (only `render_radials` produced it) and `Error::ImageSave(String)` (now
  `Error::Image`). (#174)

### Fixed

- `RenderMetadata::pixel_to_polar` no longer divides by zero for `render_cartesian` results; it returns `None`. (#174)
- `render_cartesian` and `render_vertical` metadata no longer report fake polar values (`pixels_per_km = 0`, a made-up
  `center_pixel`). (#174)
- `ColorLookupTable::new` clamps sizes below 2 to 2; the old constructors divided by zero or produced NaN for sizes 0
  and 1. (#174)
