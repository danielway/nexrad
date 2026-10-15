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

- `nexrad::chrono` re-exports `chrono` (feature `chrono`, implied by `aws`), so callers of `list_scans` and
  `download_at` do not need their own `chrono` dependency. (#176)
- `stream_with(config, sleep)`, which takes a `nexrad::data::aws::realtime::PollConfig` to set the site and retry
  policies; `stream(site, sleep)` is `stream_with(PollConfig::new(site), sleep)`. (#195)
- `stream_scans(site, sleep)` and `stream_scans_with(config, sleep)`, which yield one `Scan` per complete real-time
  volume, assembled with `assemble_volume`. Chunks before the first Start chunk are skipped, an error from the chunk
  stream is yielded and discards the partial volume, and only complete volumes are yielded. (#195)
- `nexrad::futures_util` re-exports `futures_util` (feature `aws`), for `StreamExt::next` on the real-time streams.
  (#195)
- `Error::NoScanAt { site, datetime }` (feature `aws`), returned by `download_at` when no archive volume overlaps the
  requested datetime. (#195)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` is `#[non_exhaustive]`; matches on it outside this crate need a wildcard arm. (#155)
- Features (#156, #161):
  - The `tokio` feature and the `aws-polling` feature are removed; real-time streaming is part of `aws`.
  - Sub-crates are depended on with their default features disabled; each facade feature enables what it needs.
  - `decode` implies `model`; `data` implies `decode` and `model`; `parallel` implies `data`.
  - `full` is `wasm` + `parallel`.
- `stream(site, sleep)` takes a caller-supplied sleep function as its second argument (with tokio,
  `|d| tokio::time::sleep(d)`) and no longer depends on tokio. (#161)
- `download(identifier: Identifier)` → `download(identifier: &Identifier)`. (#176)
- `load(data: &[u8])` → `load(data: Vec<u8>)`; it takes ownership of the bytes instead of copying them. `load_file` no
  longer copies the file's contents a second time. (#195)
- `download_latest(site, date)` → `download_latest(site)`, which downloads the newest archived volume: the last volume
  of the current UTC date, or of the previous UTC date when the current one has none yet. For the last volume of a
  given date, call `list_scans(site, date)` and pass the last identifier to `download`. (#195)
- `download_at(site, datetime)` returns the volume whose collection overlaps `datetime`: the last volume that started
  at or before it, looking into the previous UTC date when `datetime` precedes the date's first volume, and only if it
  started at most 15 minutes earlier. A miss returns `Error::NoScanAt` instead of `Error::NoDataAvailable`, which it
  used to return both for an empty date and for a time before the date's first volume. (#195)
- `list_scans`, `download_latest` and `download_at` see archive volume files only, so `download_latest` can no longer
  pick an `_MDM` metadata object. (#192)
- Errors from `nexrad-data` are flattened by layer (#176):
  - Decode, model and I/O errors raised inside `nexrad-data` (from `load`, `load_file`, `download*`, `stream`, or `?`
    on a `nexrad_data::Error`) become `Error::Decode`, `Error::Model` and `Error::Io` instead of
    `Error::Data(nexrad_data::Error::Decode(..))` and so on. Their `Display` text changes accordingly.
  - Every other data error is still `Error::Data`.
- `Error::NoDataAvailable` exists only with the `aws` feature, and its `date` field is a `chrono::NaiveDate` instead of
  a `String`; `Display` is unchanged. (#176)
- The wrapping variants `Model`, `Decode`, `Data`, `Render`, `Process` and `Io` no longer repeat the wrapped error in
  their `Display` text, which is the layer name alone (`decode error`); the wrapped error is still their `source()`,
  so reporters that walk the source chain print it once. (#195)

### Removed

- The model helpers, which are now methods on the model types (#172):
  - `extract_field(sweep, product)` → `sweep.field(product)`
  - `extract_fields(&scan, product)` → `scan.fields(product)`
  - `extract_first_field(&scan, product)` → `scan.first_field(product)`
  - `coordinate_system(&scan)` → `scan.coordinate_system()`
  - `coordinate_system_required(&scan)?` → `scan.coordinate_system().ok_or(/* your error */)?`
- The site registry wrappers; use `nexrad::model::meta::registry` (#172):
  - `sites()` → `registry::sites()`
  - `site(id)` → `registry::site_by_id(id)`
  - `nearest_site(lat, lon)` → `registry::nearest_site(lat, lon)`, which takes `f64` coordinates (#164)
- `Error::MissingSiteMetadata`, which only `coordinate_system_required` produced. (#172)
