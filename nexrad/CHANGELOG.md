# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.4`, the last published release. Together they make up the 1.0.0 release. The re-exported
crates (`nexrad::model`, `nexrad::decode`, `nexrad::data`, `nexrad::process`, `nexrad::render`) carry their own
breaking changes; see their CHANGELOGs.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `nexrad::chrono` re-exports `chrono` (feature `chrono`, implied by `aws`). (#176)
- `stream_with(config, sleep)`, which streams with a `PollConfig`. (#195)
- `stream_scans(site, sleep)` and `stream_scans_with(config, sleep)`, which yield one `Scan` per complete real-time
  volume; chunks before the first Start chunk are skipped, and a stream error discards the partial volume. (#195)
- `nexrad::futures_util` re-exports `futures_util` (feature `aws`), for `StreamExt` on the real-time streams. (#195)
- `Error::NoScanAt { site, datetime }` (feature `aws`), from `download_at`. (#195)
- An opt-in `image` feature enabling `nexrad-render/image` (`RenderResult::into_rgba_image`). It is not part of `wasm`
  or `full`, tracks the `image` crate's own 0.x releases and is excluded from the semver promise. (#203)

### Changed

- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- The `tokio` and `aws-polling` features are removed, sub-crates are depended on without their default features, and
  each facade feature enables the features it builds on; `full` is `wasm` + `parallel`. (#156, #161)
- `stream(site, sleep)` takes a caller-supplied sleep function and no longer depends on tokio. (#161)
- `download` takes the identifier by reference. (#176)
- `load` takes the bytes as a `Vec<u8>` instead of copying them, and `load_file` no longer copies the file twice.
  (#195)
- `download_latest(site)` downloads the newest archived volume of the current or previous UTC date. (#195)
- `download_at` returns the volume whose collection overlaps `datetime`, or `Error::NoScanAt`. (#195)
- `list_scans`, `download_latest` and `download_at` see archive volume files only. (#192)
- Errors from `nexrad-data` are flattened by layer: its decode, model and I/O errors become `Error::Decode`,
  `Error::Model` and `Error::Io`, and other data errors stay `Error::Data`. (#176)
- `Error::NoDataAvailable` requires the `aws` feature, and its `date` is a `chrono::NaiveDate`. (#176)
- The `render` feature no longer pulls in the `image` crate; `nexrad::render` results are owned RGBA8 pixel buffers.
  (#203)
- The wrapping variants `Model`, `Decode`, `Data`, `Render`, `Process` and `Io` display only their layer, with the
  wrapped error as their `source()`. (#195)
- docs.rs shows the feature each feature-gated item requires. (#196)
- The package description covers loading, downloading, decoding, processing and rendering. (#196)
- The `aws` feature's build requirements (a C compiler, and NASM on Windows x86 targets) are documented. (#196)

### Removed

- The model helpers `extract_field`, `extract_fields`, `extract_first_field`, `coordinate_system` and
  `coordinate_system_required`; use the methods on `Sweep` and `Scan`. (#172)
- The site registry wrappers `sites`, `site` and `nearest_site`; use `nexrad::model::meta::registry`. (#164, #172)
- `Error::MissingSiteMetadata`, which only `coordinate_system_required` produced. (#172)
- The `uom` feature, also dropped from `wasm` and `full`; `nexrad-model` and `nexrad-decode` no longer expose `uom`
  quantities. Use their unit-suffixed accessors. (#202)
