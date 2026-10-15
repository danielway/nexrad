# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Changes since `1.0.0-rc.7`, the last published release. Together they make up the 1.0.0 release.

See [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md) for step-by-step upgrade instructions.

### Added

- `Error` and `Result` are re-exported at the crate root (`nexrad_data::Error`), and `AwsError` with the `aws`
  feature (`nexrad_data::AwsError`); `nexrad_data::result` remains the defining module. (#158)
- `AwsError::is_transient()`, which reports whether a failure is worth retrying. (#171)
- In `aws::realtime`: `DownloadedChunk::new(identifier, chunk, attempts)`, `ChunkIdentifier::with_sequence`,
  `get_latest_volume_from_hint(site, hint)`, `ChunkIterator::config()`, and getters on `RetryPolicy`
  (`max_attempts()`, `initial_delay()`, `backoff_multiplier()`, `max_delay()`) and `PollConfig` (`site()`,
  `download_policy()`, `discovery_policy()`). (#167)
- `volume::File::coverage_pattern_message()`, which decodes the file's volume coverage pattern (message type 5) as an
  owned `nexrad_decode` message and returns `Error::MissingCoveragePattern` when the file has none. Use it on a
  real-time `Chunk::Start`'s file to read the volume's VCP. (#183)
- `aws::archive::list_files_with_sizes(site, date)`, which returns each volume file's `ListedFile` (its `Identifier`
  and compressed size in bytes) from the same listing request `list_files` makes. File size tracks how much echo a
  volume holds, so it works as a cheap storm-activity signal without downloading anything. (#186)
- `aws::archive::Identifier::version()` (the `Vnn` suffix), `is_metadata()` (`_MDM` objects) and `is_gzip()` (`.gz`
  objects), plus `FromStr`, `Display` (the name) and `Debug`. (#192)
- `aws::archive::ListedFile` implements `Debug` and `PartialOrd`/`Ord`, ordering by identifier and then size. (#192)
- `AwsError::InvalidIdentifier { name, reason }`, `AwsError::InvalidVolumeIndex { index }` and
  `AwsError::HttpClient(Box<dyn std::error::Error + Send + Sync>)`; none is transient. (#192)

### Changed

- The minimum supported Rust version is now 1.88 (it was declared as 1.75, which was already incorrect). (#157)
- `Error` and `AwsError` are `#[non_exhaustive]`; matches on them outside this crate need a wildcard arm. (#155)
- Features (#156, #161):
  - The `nexrad-model` feature is renamed to `model`.
  - The `aws-polling` feature is removed; real-time streaming is part of `aws`.
  - Optional dependencies use `dep:` syntax, so `futures`, `nexrad-model`, `once_cell`, `rayon`, `reqwest`, `tokio`
    and `xml` are no longer features. Use `aws`, `model` or `parallel`.
  - `nexrad-decode`'s default features are no longer enabled through this crate, so its `uom` feature is no longer
    turned on transitively.
- Acronyms in names follow UpperCamelCase: `AWSError` → `AwsError`, `Error::AWS` → `Error::Aws`. (#160)
- `AwsError` moved from `nexrad_data::result::aws::AwsError` to `nexrad_data::result::AwsError` (the `result::aws`
  module is removed); `nexrad_data::AwsError` also works. (#171)
- `AwsError::S3ListObjects`, `S3GetObjectRequest` and `S3Streaming` hold a
  `Box<dyn std::error::Error + Send + Sync + 'static>` instead of a `reqwest::Error`; `source()` returns it. (#171)
- `Error` and `AwsError` no longer implement `UnwindSafe`/`RefUnwindSafe`, because the boxed error sources do not;
  wrap them in `std::panic::AssertUnwindSafe` where that matters. (#171)
- `aws::realtime::chunk_stream(config, sleep)` takes a caller-supplied sleep function as its second argument (with
  tokio, `|d| tokio::time::sleep(d)`) and no longer depends on tokio, so it builds for WASM. (#161)
- `aws::realtime::Chunk<'a>` → `Chunk`; its `IntermediateOrEnd` variant holds a `volume::Record<'static>`.
  `DownloadedChunk::chunk` and `assemble_volume(chunks: impl IntoIterator<Item = Chunk>)` follow. (#167)
- `aws::realtime::download_chunk(site, &id)` returns `Result<DownloadedChunk>` instead of
  `Result<(ChunkIdentifier, Chunk)>`; read `d.identifier` and `d.chunk`. (#167)
- `aws::realtime::ChunkIterator::start_with_policies(site, download, discovery)` → `start_with(config: PollConfig)`,
  and `from_chunk(site, chunk_id, download, discovery)` → `from_chunk(config, chunk_id)`. Build the config with
  `PollConfig::new(site).with_download_policy(..).with_discovery_policy(..)`. (#167)
- `aws::realtime::ChunkIterator::download_policy()` / `discovery_policy()` → `config().download_policy()` /
  `config().discovery_policy()`, and `project_remaining_scan()` → `projected_volume_end_time()`. (#167)
- `aws::realtime::RetryPolicy` and `PollConfig` fields are private; use the getters and the existing `with_*` setters
  and constructors. (#167)
- `aws::realtime::RetryPolicy` delays (`with_initial_delay`, `with_max_delay`, `initial_delay()`, `max_delay()`) and
  `ChunkIterator::time_until_next()` use `std::time::Duration` instead of `chrono::Duration`. (#167)
- `aws::realtime::DownloadedChunk`, `ChunkIteratorInit` and `LatestVolumeResult` are `#[non_exhaustive]`; their
  fields stay readable, but struct literals and exhaustive patterns outside the crate no longer compile. (#167)
- `aws::realtime::ChunkIdentifier::new` and `from_name` take `impl Into<String>` for `site` (and `name`); pass
  `"KDMX"` rather than `"KDMX".into()`. (#167)
- `volume::Record::decompress(&self)` returns `Result<Record<'static>>`. (#171)
- `volume::Header::tape_filename()`, `extension_number()` and `icao_of_radar()` return `Option<&str>` instead of
  `Option<String>`. (#171)
- `volume::Header` no longer implements zerocopy traits (`FromBytes`, `FromZeros`, `Immutable`, `KnownLayout`,
  `TryFromBytes`) and is no longer `#[repr(C)]`; `volume::File::header()` returns `Option<Header>` by value instead of
  `Option<&Header>`, and `Header::SIZE` replaces `size_of::<Header>()` for the header length. (#181)
- `aws::archive::list_files(site, date: NaiveDate)` takes the date by value, and
  `aws::archive::download_file(identifier: &Identifier)` takes the identifier by reference. (#171)
- `aws::realtime::list_chunks_in_volume` reads every listing page and returns only the chunks of the newest scan in a
  reused volume directory, sorted by sequence. `ChunkIterator` waits instead of moving into a next volume that still holds an older scan. (#149)
- `aws::realtime::ChunkIterator` and `chunk_stream` consume each new volume from its Start chunk: at rollover they
  yield sequence 1 (the volume header and VCP) and then every later chunk in sequence order, instead of jumping to the
  newest chunk already published in the next volume. Every volume's chunks can now be passed to `assemble_volume`, and
  the VCP and chunk timing are refreshed when a new volume changes VCP. (#183)
- `aws::archive::Identifier::new(String)` → `Identifier::parse(&str) -> Result<Identifier>`, which validates the
  archive name grammar (`SSSSYYYYMMDD_HHMMSS[_Vnn][_MDM][.gz]`, UTC) and returns `AwsError::InvalidIdentifier` for
  other names. `site()` returns `&str` and `date_time()` returns `DateTime<Utc>` instead of `Option`s. Equality,
  ordering and hashing use the name only. (#192)
- `aws::archive::list_files` and `list_files_with_sizes` return volume files only, skipping `_MDM` metadata objects and
  objects whose names do not parse. (#192)
- `aws::realtime::VolumeIndex::new(index)` returns `Result<VolumeIndex>` and rejects indexes outside `1..=999` with
  `AwsError::InvalidVolumeIndex`. (#192)
- `aws::realtime::list_chunks_in_volume(site, volume, max_keys)` → `list_chunks_in_volume(site, volume)`; it always
  returns every chunk of the newest scan. (#192)
- `aws::realtime::chunk_stream` yields the chunks it downloads at startup: the volume's Start chunk when joining
  mid-volume, then the latest chunk, then the chunks after it. (#192)
- `aws::realtime::ChunkIterator::try_next` makes a single attempt per call, including at volume rollover, and returns
  `Ok(None)` until the next chunk or volume is available; request errors propagate instead of being retried or
  swallowed. `PollConfig`'s retry policies apply to `chunk_stream` only. (#192)
- `AwsError::S3GetObject(Option<String>)` → `S3GetObject { status: u16, body: Option<String> }`. (#192)
- Volume assembly is delegated to `nexrad-decode`'s `ScanBuilder` (#169):
  - `volume::File::scan` includes legacy (type 1) radials, consistent with `Record::radials()`;
    `assemble_volume` accepts type 1 messages too.
  - The scan's `Site::identifier` comes from the radar identifier in the type 31 message header instead of the
    Archive II volume header ICAO.
- `volume::Record::messages()`, `Record::radials()`, `File::scan()`, `File::coverage_pattern_message()` and
  `aws::realtime::assemble_volume` fail with `Error::Decode` on the first message that cannot be decoded, following
  `nexrad-decode`'s strict `decode_messages`; they previously skipped such messages. (#190)

### Removed

- From `aws::realtime`, the timing internals: `ChunkTimingModel`, `ChunkTimingStats`, `ChunkCharacteristics`,
  `ElevationChunkMapper`, `ChunkMetadata`, `estimate_chunk_availability_time` (use
  `ChunkIterator::next_expected_time()` / `time_until_next()`), `estimate_chunk_processing_time`,
  `project_scan_timing`, `project_full_scan_timing`, `ScanTimingProjection` and `ChunkProjection` (use
  `ChunkIterator::projected_volume_end_time()`). (#167)
- From `aws::realtime`, `RetryState` (configure retries with `RetryPolicy`), `NextChunk` and
  `ChunkIdentifier::next_chunk` (`ChunkIterator` and `chunk_stream` handle sequencing). (#167)
- `ChunkIterator::timing_stats()`, `timing_stats_mut()`, `elevation_mapper()`, `chunk_metadata()` and
  `all_chunk_metadata()`, which exposed the internals above. (#167)
- `AwsError::PollingAsync`, which was never constructed. (#171)
- `Error::Decompression(bzip2::Error)` and its `From<bzip2::Error>` impl, which were never constructed; bzip2 failures
  surface as `Error::Io`. (#171)
- `volume::split_compressed_records`; use `volume::File::new(bytes).records()`. (#171)
- `AwsError::DateTime(String)` and `AwsError::InvalidSiteIdentifier(String)`; a parsed `Identifier` always has a site
  and date, and invalid names fail in `Identifier::parse` with `AwsError::InvalidIdentifier`. (#192)

### Fixed

- `aws::realtime::get_latest_volume` no longer returns an older volume when the bucket's volume numbers wrap around.
  (#149, #154)
- `volume::File::records()` returns `Error::TruncatedRecord` instead of panicking when the data is shorter than the
  24-byte volume header. (#171)
- Building the shared HTTP client no longer panics on failure; requests return `AwsError::HttpClient` instead, and
  `once_cell` is no longer a dependency. (#192)
- `aws::realtime::chunk_stream` no longer caps its startup listing at 100 chunks, so in a longer volume it joins at the
  latest chunk. (#192)
- `aws::realtime`: after a volume's End chunk, `ChunkIterator::next_expected_time` and `chunk_stream` expect the next
  volume's Start chunk after the 8.5 s inter-volume gap instead of the 4 to 11 s elevation-based fallback. (#192)
- `aws::realtime`: after a volume's Start chunk, `ChunkIterator::next_expected_time` and `chunk_stream` expect
  sequence 2 about 1.5 s later instead of after the 8.5 s inter-volume gap. (#184)
- `aws::realtime`: real-time timing statistics keep samples for the first chunk of a sweep, which include the
  inter-sweep transition, apart from samples for chunks within a sweep. (#184)
- `aws::realtime`: `ChunkIterator` and `chunk_stream` record the interval into each chunk under that chunk's
  characteristics and read it back under the same key when predicting the chunk, so learned timings are applied at
  sweep boundaries instead of falling back to the physics model. (#184)
- The hop from the Start chunk to sequence 2 is no longer recorded in the timing statistics; its estimate is a
  fixed gap, so the sample only skewed the first-in-sweep bucket of the first elevation. (#184)
