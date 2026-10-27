# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.4`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root; the `result` module remains. (#158)
- `Projection` (`Polar`, `Geographic`, `Vertical`), read with `RenderMetadata::projection()`; the enum and its
  variants are `#[non_exhaustive]`. (#174, #182)
- `pub use image;`; an `image` major version bump is a `nexrad-render` major version bump. (#174)
- `ColorLookupTable::new(&ColorScale, min, max, size) -> Result<ColorLookupTable>`, replacing `from_scale` and
  `from_color_scale`. (#174, #194)
- `Error::Image(image::ImageError)`, from `RenderResult::save` and `write_to`; it displays only its layer, with the
  `ImageError` as its `source()`. (#174, #195)
- `RenderOptions::with_value_range(min, max)` and `value_range()`. (#194)
- `RenderOptions::with_status_color(GateStatus, Color)` and `status_color(GateStatus)`, for gates with a non-`Valid`
  status. (#194)
- `RenderOptions` implements `PartialEq`. (#194)
- `ColorLookupTable::entries()`, `min_value()`, `max_value()`, `len()` and `is_empty()`. (#194)
- `Color::from_rgba8`, conversions to and from `[u8; 4]`, and `Default for Color` (`Color::TRANSPARENT`). (#194)
- Public `ColorScaleLevel` fields, so `DiscreteColorScale::levels()` can drive a legend. (#194)
- `RenderMetadata::pixel_to_vertical`, `vertical_to_pixel`, `query_vertical` and `query_cartesian`. (#194)
- `RenderResult::write_to(&mut impl Write, image::ImageFormat)`, which encodes without touching the filesystem. (#194)
- `Error::InvalidRange`, `Error::InvalidTableSize`, `Error::InvalidExtent` and `Error::MissingCoordinateSystem`. (#194)

### Changed

- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- Angle accessors use the `nexrad-model` stems `azimuth_angle_degrees` and `elevation_angle_degrees`, and polar angles
  are `f64`. (#164, #182)
- `default_scale(Product)` returns a `ColorScale` instead of a `DiscreteColorScale`. (#174)
- Image dimensions are `u32`. (#174)
- `Error` no longer implements `UnwindSafe`/`RefUnwindSafe`, because `image::ImageError` does not. (#174)
- `Error::NoRadials` → `Error::EmptyField`, also for Cartesian and vertical fields with no rows or columns;
  `Error::InvalidDimensions` also covers a zero or overflowing size. (#174)
- Polar-only `RenderMetadata` accessors and the point queries return `Option`, `Some` only for `Projection::Polar`.
  (#174)
- `RenderMetadata::pixel_to_geo` and `geo_to_pixel` map linearly through the extent for `render_cartesian` results and
  return `None` for `render_vertical` results. (#174)
- `RenderResult::new` is crate-private. (#174)
- `PointQuery` is `#[non_exhaustive]`. (#182)
- Every renderer builds its color lookup table over `RenderOptions::value_range()`, else the product's range, else the
  field's valid-value range, else `-32..95`. (#174, #194)
- `RenderOptions` takes and returns its background as a `Color`. (#194)
- `query_pixel`, `query_polar` and `query_geo` moved from `RenderResult` to `RenderMetadata`. (#194)
- `RenderOptions::with_extent` makes `render_sweep` draw onto a latitude/longitude grid covering that extent. (#194)
- `Color::to_rgba8` clamps and rounds to the nearest byte instead of truncating. (#194)
- The `image` dependency enables only its `png` feature. (#194)
- The published crate leaves out the benchmark, which needs files from outside it. (#196)
- Documentation only: the README states the supported data scope and its code blocks compile as doc tests; the color
  scale docs list value ranges and colors without interpreting them. (#PR)

### Removed

- `render_radials` and `render_radials_default`; build a `SweepField` and call `render_sweep`. (#174)
- `default_color_scale(Product)`; use `default_scale(Product)`. (#174)
- `ColorLookupTable::from_scale` and `from_color_scale`; use `ColorLookupTable::new`. (#174)
- `Error::ProductNotFound` and `Error::ImageSave(String)`; saving fails with `Error::Image`. (#174)
- `RenderOptions::transparent()` and `bilinear()`. (#194)
- The `nexrad_render::Product` re-export; use `nexrad_model::data::Product`. (#194)

### Fixed

- `RenderMetadata::pixel_to_polar` no longer divides by zero for `render_cartesian` results. (#174)
- `render_cartesian` and `render_vertical` metadata no longer report fake polar values. (#174)
- Color lookup tables are no longer built with NaN entries or reversed for degenerate sizes and ranges. (#174, #194)
- `ColorLookupTable::color` returns a discrete scale's exact color for every in-range value at any table size, and
  continuous scales use the nearest entry instead of the one below. (#194)
- `RenderMetadata::geo_extent()` matches the pixels when `RenderOptions::with_extent` is set. (#194)
- `RenderMetadata::query_pixel`, `query_polar` and `query_geo` on a sector or partial sweep return `None` beyond the
  covered azimuths. (#185)
