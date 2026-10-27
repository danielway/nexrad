# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.7`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root, and `AwsError` with the `aws` feature; the `result` module
  remains. (#158)
- `AwsError::is_transient()`, which reports whether a failure is worth retrying: transport failures, timeouts and the
  HTTP statuses 408, 429 and 5xx are transient, and every other 4xx status is terminal. (#171, #206)
- In `aws::realtime`: `DownloadedChunk::new`, `ChunkIdentifier::with_sequence`, `get_latest_volume_from_hint`,
  `ChunkIterator::config()`, and getters on `RetryPolicy` and `PollConfig`. (#167)
- `volume::File::coverage_pattern_message()`, which decodes a file's volume coverage pattern, for example a real-time
  `Chunk::Start`'s, or returns `Error::MissingCoveragePattern`. (#183)
- `aws::archive::list_files_with_sizes` and `ListedFile`: each volume file's identifier and compressed size from the
  same listing request, a cheap storm-activity signal. (#186, #192)
- `aws::archive::Identifier::version()`, `is_model_data()` and `is_gzip()`, plus `FromStr`, `Display` and `Debug`.
  `is_model_data()` marks `_MDM` objects, which hold the Model Data message (type 29). (#192, #206)
- `volume::Record::iter_radials()` and `volume::RadialIter` (feature `model`), which yield one `Result<Radial>` per
  radar data message, so a radial that does not convert can be skipped while the rest are kept. (#206)
- `AwsError::InvalidIdentifier`, `AwsError::InvalidVolumeIndex` and `AwsError::HttpClient`, none of them transient.
  (#192)

### Changed

- The minimum supported Rust version is 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` and `AwsError` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard arm. (#155)
- The `nexrad-model` feature is renamed `model`, `aws-polling` is folded into `aws`, optional dependencies are no
  longer features, and `nexrad-decode`'s default features (including `uom`) are no longer enabled. (#156, #161)
- `result::aws::AWSError` → `result::AwsError`, also at the crate root, and `Error::AWS` → `Error::Aws`. (#160, #171)
- `aws::realtime::VolumeIndex::as_number()` → `number()`, matching the other `number()` accessors. (#206)
- `AwsError::S3ListObjects`, `S3GetObjectRequest` and `S3Streaming` box the HTTP error instead of holding a
  `reqwest::Error`. (#171)
- `Error` and `AwsError` no longer implement `UnwindSafe`/`RefUnwindSafe`. (#171)
- `aws::realtime::chunk_stream(config, sleep)` takes a caller-supplied sleep function and no longer depends on tokio.
  (#161)
- `aws::realtime::Chunk` has no lifetime parameter. (#167)
- `aws::realtime::download_chunk` returns `Result<DownloadedChunk>`. (#167)
- `ChunkIterator::start_with` and `from_chunk` take a `PollConfig` instead of a site and two retry policies. (#167)
- `ChunkIterator`'s policies are read through `config()`, and `project_remaining_scan()` is
  `projected_volume_end_time()`. (#167)
- `RetryPolicy` and `PollConfig` fields are private. (#167)
- Real-time delays use `std::time::Duration` instead of `chrono::Duration`. (#167)
- `DownloadedChunk`, `ChunkIteratorInit` and `LatestVolumeResult` are `#[non_exhaustive]`. (#167)
- `ChunkIdentifier::new` and `from_name` take `impl Into<String>`. (#167)
- `volume::Record::decompress` returns `Result<Record<'static>>`. (#171)
- `volume::Header`'s string accessors return `Option<&str>`. (#171)
- `volume::Header` is no longer `#[repr(C)]` or zerocopy; `File::header()` returns it by value, and `Header::SIZE` is
  its length. (#181)
- `aws::archive::list_files` takes the date by value and `download_file` the identifier by reference. (#171)
- `list_chunks_in_volume` drops its `max_keys` cap and returns every chunk of the newest scan in a reused volume
  directory, and `ChunkIterator` waits instead of entering a volume that still holds an older scan. (#149, #192)
- `ChunkIterator` and `chunk_stream` consume each new volume from its Start chunk, so every volume can be passed to
  `assemble_volume`, and refresh the VCP and chunk timing when a new volume changes VCP. (#183)
- `Identifier::new` → `Identifier::parse`, which validates the archive name; `site()` and `date_time()` no longer
  return `Option`s, and equality, ordering and hashing use the name. (#192)
- `aws::archive::list_files` returns volume files only. (#192)
- `aws::realtime::VolumeIndex::new` returns `Result` and rejects indexes outside `1..=999`. (#192)
- `aws::realtime::chunk_stream` yields the chunks it downloads at startup. (#192)
- `ChunkIterator::try_next` makes a single attempt per call and propagates request errors; `PollConfig`'s retry
  policies apply to `chunk_stream` only. (#192)
- `AwsError::S3GetObject` carries the HTTP `status` and `body`. (#192)
- The struct-like variants of `Error` and `AwsError` (`TruncatedRecord`, `InvalidRecordSize`, `InvalidIdentifier`,
  `InvalidVolumeIndex`, `S3GetObject`) are `#[non_exhaustive]`: patterns outside this crate need `..`. (#206)
- `aws::realtime::ChunkType` is `#[non_exhaustive]`; the Archive II ICD defines a fourth LDM key type, `M`. (#206)
- `volume::Record::decompress` returns an uncompressed record unchanged instead of failing, as `File::decompress`
  does. (#206)
- `Error::CompressedData` → `Error::CompressedRecord`, beside `Error::CompressedFile`. (#206)
- `aws::realtime::ChunkIdentifier::new` takes the date-time prefix as `DateTime<Utc>` (fractional seconds are dropped),
  `date_time_prefix()` returns `DateTime<Utc>` by value, and `volume()` returns `VolumeIndex` by value. (#206)
- `aws::realtime::download_chunk` takes only the chunk identifier, which carries the site. (#206)
- `aws::realtime::list_chunks_in_volume` skips, with a `log` warning, object keys that do not parse as chunk names
  instead of failing the listing. (#206)
- The native HTTP client applies a 30 s connect timeout, a 60 s read timeout and a 300 s request timeout. They do not
  apply on wasm32, where the browser's `fetch` limits apply. (#206)
- Volume assembly is delegated to `nexrad-decode`'s `ScanBuilder`: `File::scan` and `assemble_volume` include legacy
  (type 1) radials, and a scan's site identifier comes from the type 31 message header. (#169)
- `Record::messages()`, `Record::radials()`, `File::scan()`, `File::coverage_pattern_message()` and
  `assemble_volume` fail with `Error::Decode` on the first undecodable message instead of skipping it. (#190)
- docs.rs shows the feature each feature-gated item requires. (#196)
- The published crate leaves out the tests and benchmarks that need files from outside it, the fuzz targets and
  the `trim_fixture` maintainer example. (#196)
- The `aws` feature's build requirements (a C compiler, and NASM on Windows x86 targets) are documented. (#196)
- `volume::File::scan()` and `File::coverage_pattern_message()` inflate a gzip-wrapped file themselves instead of
  returning `Error::CompressedFile`. `File::records()`, which borrows its records from the file, still needs
  `File::decompress()` first. (#208)
- `volume::File::scan()` and `aws::realtime::assemble_volume` return a scan whose `coverage_pattern()` is `None` when
  no type 5 message was seen, such as for a legacy volume or chunks without a Start chunk, with the coverage pattern
  number taken from the radials. They fail only when no coverage pattern number is found, with `Error::Decode`
  carrying `MissingCoveragePatternNumber`, instead of `Error::MissingCoveragePattern`, which
  `File::coverage_pattern_message()` still returns. (#208)

### Removed

- The `aws::realtime` timing internals (`ChunkTimingModel`, `ElevationChunkMapper`, the estimate and projection
  functions and their types); `ChunkIterator` exposes the estimates. (#167)
- `aws::realtime::RetryState`, `NextChunk` and `ChunkIdentifier::next_chunk`. (#167)
- `ChunkIterator`'s timing and metadata accessors (`timing_stats()`, `chunk_metadata()`, ...). (#167)
- `AwsError::PollingAsync`, which was never constructed. (#171)
- `Error::Decompression` and `From<bzip2::Error>`, which were never constructed; bzip2 failures are `Error::Io`. (#171)
- `volume::split_compressed_records`; use `volume::File::records()`. (#171)
- `AwsError::DateTime` and `AwsError::InvalidSiteIdentifier`; invalid names fail in `Identifier::parse`. (#192)
- `Error::UncompressedData`; `Record::decompress` no longer fails on an uncompressed record. (#206)

### Fixed

- `aws::realtime::get_latest_volume` no longer returns an older volume when volume numbers wrap around. (#149, #154)
- `volume::File::records()` returns `Error::TruncatedRecord` instead of panicking on data shorter than the volume
  header. (#171)
- Building the shared HTTP client no longer panics; requests return `AwsError::HttpClient`, and `once_cell` is dropped.
  (#192)
- `aws::realtime::chunk_stream` no longer caps its startup listing at 100 chunks, so it joins long volumes at the
  latest chunk. (#192)
- `aws::realtime::get_latest_volume` searches volume 999. (#192)
- `aws::realtime::ChunkIterator::from_chunk` builds an iterator whose `try_next` works: the first call loads the VCP
  from the current volume's Start chunk, where it always failed with `FailedToDetermineNextChunk`. (#206)
- `aws::realtime`: after an End chunk, the next volume's Start chunk is expected after the 8.5 s inter-volume gap.
  (#192)
- `aws::realtime`: after a Start chunk, sequence 2 is expected about 1.5 s later instead of after the inter-volume gap.
  (#184)
- `aws::realtime`: timing statistics keep samples for a sweep's first chunk apart from those within a sweep. (#184)
- `aws::realtime`: learned chunk timings are recorded and read under the same key, so they apply at sweep boundaries.
  (#184)
- `aws::realtime`: the hop from the Start chunk to sequence 2 is no longer recorded in the timing statistics. (#184)
