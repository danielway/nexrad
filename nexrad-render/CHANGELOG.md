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
- `ColorLookupTable::new(&ColorScale, min, max, size) -> Result<ColorLookupTable>`, replacing `from_scale` and
  `from_color_scale`. (#174, #194)
- `RenderResult::width()`, `height()`, `pixels() -> &[u8]`, `pixel(x, y)`, `into_pixels() -> Vec<u8>` and
  `into_parts()`: the result owns its pixels as 8-bit RGBA, row-major with no row padding. (#203)
- `RenderResult::write_png(impl Write)` and `save_png(path)`, which encode an 8-bit RGBA PNG with a private encoder.
  (#203)
- `Error::Io(std::io::Error)` and `Error::Encode(Box<dyn Error + Send + Sync>)`, from `write_png` and `save_png`;
  `Encode` displays only its layer, with the encoder's error as its `source()`. (#203)
- An opt-in `image` feature adding `RenderResult::into_rgba_image() -> image::RgbaImage`. It tracks the `image`
  crate's own 0.x releases and is excluded from the semver promise. (#203)
- `RenderOptions::with_value_range(min, max)` and `value_range()`. (#194)
- `RenderOptions::with_status_color(GateStatus, Color)` and `status_color(GateStatus)`, for gates with a non-`Valid`
  status. (#194)
- `RenderOptions` implements `PartialEq`. (#194)
- `ColorLookupTable::entries()`, `min_value()`, `max_value()`, `len()` and `is_empty()`. (#194)
- `Color::from_rgba8`, conversions to and from `[u8; 4]`, and `Default for Color` (`Color::TRANSPARENT`). (#194)
- Public `ColorScaleLevel` fields, so `DiscreteColorScale::levels()` can drive a legend. (#194)
- `RenderMetadata::pixel_to_vertical`, `vertical_to_pixel`, `query_vertical` and `query_cartesian`. (#194)
- `Error::InvalidRange`, `Error::InvalidTableSize`, `Error::InvalidExtent` and `Error::MissingCoordinateSystem`. (#194)

### Changed

- `nws_reflectivity_scale()` is `reflectivity_scale()`, matching `velocity_scale()` and the other scale functions; the
  name claimed an NWS source that nothing cites. (#210)
- `DiscreteColorScale::new` and `ContinuousColorScale::new` return `Result` and reject an empty list or a value that
  is not finite, with the new `Error::InvalidColorScale { reason }`. (#210)
- `RenderMetadata::geo_to_pixel` and `query_geo` return `None` for a polar render when no beam at the sweep's
  elevation passes over the point. (#210)
- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- Angle accessors use the `nexrad-model` stems `azimuth_angle_degrees` and `elevation_angle_degrees`, and polar angles
  are `f64`. (#164, #182)
- `default_scale(Product)` returns a `ColorScale` instead of a `DiscreteColorScale`. (#174)
- Image dimensions are `u32`. (#174)
- `Error` no longer implements `UnwindSafe`/`RefUnwindSafe`, because the boxed encoder error in `Error::Encode` does
  not. (#174, #203)
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
- PNG encoding uses the `png` crate as a private dependency; `image` is a dependency only with the `image` feature.
  (#194, #203)
- The published crate leaves out the benchmark, which needs files from outside it. (#196)
- `ColorScale` and `Interpolation` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard arm.
  (#207)
- A gate whose `GateStatus` is one a later `nexrad-model` release adds is drawn with the `NoData` status color.
  (#211)

### Removed

- `render_radials` and `render_radials_default`; build a `SweepField` and call `render_sweep`. (#174)
- `default_color_scale(Product)`; use `default_scale(Product)`. (#174)
- `ColorLookupTable::from_scale` and `from_color_scale`; use `ColorLookupTable::new`. (#174)
- `Error::ProductNotFound` and `Error::ImageSave(String)`; saving fails with `Error::Io` or `Error::Encode`. (#174,
  #203)
- `image` types from the public API: the `RgbaImage` re-export, `RenderResult::image()`, `into_image()` and `save()`.
  Use `pixels()`, `into_pixels()` and `save_png()`, or `into_rgba_image()` with the `image` feature. (#203)
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
