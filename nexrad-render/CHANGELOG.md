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
- `RenderOptions::with_value_range(min, max)` and `value_range()`: the range the color lookup table is built over.
  (#194)
- `RenderOptions::with_status_color(GateStatus, Color)` and `status_color(GateStatus)`: how gates with a non-`Valid`
  status are drawn (default: the background), for example purple for `GateStatus::RangeFolded`. (#194)
- `RenderOptions` implements `PartialEq`. (#194)
- `ColorLookupTable::entries()` (the RGBA table, for example for a 1-D GPU texture), `min_value()`, `max_value()`,
  `len()` and `is_empty()`. (#194)
- `Color::from_rgba8`, `From<[u8; 4]> for Color`, `From<Color> for [u8; 4]` and `Default for Color`
  (`Color::TRANSPARENT`). (#194)
- `ColorScaleLevel` has public `value` and `color` fields, so `DiscreteColorScale::levels()` can drive a legend. (#194)
- `RenderMetadata::pixel_to_vertical`, `vertical_to_pixel`, `query_vertical(&VerticalField, x, y)` and
  `query_cartesian(&CartesianField, x, y)`. (#194)
- `RenderResult::write_to(&mut impl Write, image::ImageFormat)` encodes the image without touching the filesystem.
  (#194)
- `Error::InvalidRange { min, max }`, `Error::InvalidTableSize { size }`, `Error::InvalidExtent` and
  `Error::MissingCoordinateSystem`. (#194)

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
- Every renderer builds its color lookup table over `RenderOptions::value_range()` when set, else the field's product
  range (`Product::value_range`), else the field's valid-value range, else `-32..95`; a field with no valid data
  falls back to its product's range before the reflectivity default. (#174, #194)
- `ColorLookupTable::new` returns `Result<ColorLookupTable>`: `Error::InvalidRange` unless `min` and `max` are finite
  with `min < max`, `Error::InvalidTableSize` for sizes below 2 (previously clamped to 2). (#194)
- `RenderOptions::with_background` takes a `Color` and `background()` returns `Option<Color>` (`None` when fully
  transparent) instead of `[u8; 4]`. (#194)
- `query_pixel`, `query_polar` and `query_geo` moved from `RenderResult` to `RenderMetadata`, with the same signatures.
  (#194)
- `RenderOptions::with_extent` makes `render_sweep` draw the sweep onto a latitude/longitude grid covering that extent
  (`Projection::Geographic`, requiring `with_coord_system`); `render_cartesian` and `render_vertical` ignore it. (#194)
- `Color::to_rgba8` clamps components to `0.0..=1.0` and rounds to the nearest byte instead of truncating. (#194)
- The `image` dependency enables only its `png` feature; `save` and `write_to` support other formats once the consumer
  enables the matching feature on its own `image` dependency. (#194)
- `Error::Image`'s `Display` text is `image error`, without the wrapped `image::ImageError`'s message; that error is
  its `source()`, so reporters that walk the source chain print it once. (#195)
- The published crate leaves out the benchmark, which needs files from outside it.

### Removed

- `render_radials` and `render_radials_default`. Build a field with `sweep.field(product)` or
  `SweepField::from_radials(radials, product)`, then call `render_sweep(&field, &scale, &options)`. (#174)
- `default_color_scale(Product)`; use `default_scale(Product)`. (#174)
- `ColorLookupTable::from_scale` and `ColorLookupTable::from_color_scale`; use `ColorLookupTable::new`, converting a
  `DiscreteColorScale` with `.into()`. (#174)
- `Error::ProductNotFound` (only `render_radials` produced it) and `Error::ImageSave(String)` (now
  `Error::Image`). (#174)
- `RenderOptions::transparent()` and `bilinear()`; use `with_background(Color::TRANSPARENT)` and
  `with_interpolation(Interpolation::Bilinear)`. (#194)
- The `nexrad_render::Product` re-export; use `nexrad_model::data::Product`. (#194)

### Fixed

- `RenderMetadata::pixel_to_polar` no longer divides by zero for `render_cartesian` results; it returns `None`. (#174)
- `render_cartesian` and `render_vertical` metadata no longer report fake polar values (`pixels_per_km = 0`, a made-up
  `center_pixel`). (#174)
- `ColorLookupTable::new` no longer divides by zero or produces NaN for sizes 0 and 1. (#174)
- `ColorLookupTable::new` no longer maps every value to NaN for `min == max` or silently reverses for `min > max`.
  (#194)
- `ColorLookupTable::color` returns a discrete scale's exact color for every in-range value at any table size; a value
  on a threshold (for example 20 dBZ on `nws_reflectivity_scale`) used to get the color of the level below. Continuous
  scales use the nearest entry instead of the one below. (#194)
- `RenderMetadata::geo_extent()` matches the pixels when `RenderOptions::with_extent` is set. (#194)
- `RenderResult::query_pixel`, `query_polar` and `query_geo` on a sector or partial sweep return `None` beyond the
  covered azimuths instead of the value at the far end across north, through the `SweepField::value_at_polar` fix.
  (#185)
