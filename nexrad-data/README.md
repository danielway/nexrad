# NEXRAD Data

[![Crates.io](https://img.shields.io/crates/v/nexrad-data)](https://crates.io/crates/nexrad-data)
[![Docs.rs](https://docs.rs/nexrad-data/badge.svg)](https://docs.rs/nexrad-data)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Provides structure definitions and decoding functions for NEXRAD Archive II volume files, along with functions for
downloading both archival and real-time data from open cloud providers like AWS OpenData.

## Volume Definitions

The `nexrad_data::volume` module provides model definitions for the NEXRAD Archive II volume file format described in
the Radar Operations Center's ICD 2620010J for the Archive II/User (Build 23.0, June 25, 2024). A `volume::File`
can be constructed with archive or real-time data. It can decode the archive volume header and provide access to LDM
`volume::Record`s which can be decompressed and decoded into a series of messages.

With the `model` feature, `volume::File::scan` decompresses and decodes every record and assembles the result into a
`nexrad_model::data::Scan`. The conversion itself is done by `nexrad-decode`'s `ScanBuilder`; this crate only handles
I/O and decompression.

```rust,ignore
use nexrad_data::volume::File;

// Older archive files are gzip-wrapped; `decompress` unwraps them and returns other files as-is
let file = File::new(std::fs::read("KTLX20130520_201643_V06.ar2v")?).decompress()?;
let scan = file.scan()?;
println!("{} sweeps", scan.sweeps().len());
```

## AWS Open Data

NOAA uploads archive and real-time NEXRAD data to AWS Open Data S3 buckets which are publicly available. The
`nexrad_data::aws` module provides functions for listing and downloading NEXRAD data from these buckets.

### Archive Data

Historical volumes are archived by date and radar site in the `unidata-nexrad-level2` bucket. The
`nexrad_data::aws::archive` module provides functions for accessing these volumes. The `archive::list_files` function
queries volume files for a given UTC date and radar site, returning a validated identifier for each volume
(metadata objects are skipped). The `archive::download_file`
function downloads a volume file by its identifier.

### Real-Time Data

Real-time volume data is uploaded in chunks to the `unidata-nexrad-level2-chunks` bucket. 999 volume directories are
rotated through with chunks being added to each directory until they comprise a full volume. The
`nexrad_data::aws::realtime` module provides functions for accessing these chunks. The `realtime::list_chunks_in_volume`
function queries a volume for the chunks of its newest scan, returning identifiers in sequence order. The `realtime::download_chunk` function
downloads a chunk by its identifier. The `realtime::get_latest_volume` function can be used to identify which of the 999
volume directories contain the latest data, and `realtime::get_latest_volume_from_hint` does the same in a few calls
when starting from a previously seen volume. The `realtime::chunk_stream` function returns an async stream that
continuously yields chunks as they become available; it is runtime-agnostic and takes a caller-supplied sleep function
(with tokio, `chunk_stream(config, |d| tokio::time::sleep(d))`). For caller-controlled scheduling,
`realtime::ChunkIterator` provides a pull-based interface with manual timing control: `try_next` makes a single
attempt and returns `Ok(None)` until the next chunk is available, and the `next_expected_time` and `time_until_next`
methods estimate when the next chunk becomes available for download from S3. With the `model`
feature, `realtime::assemble_volume` turns a volume's chunks into a `Scan`.

## Errors

`Error` and `Result` are re-exported at the crate root, as is `AwsError` with the `aws` feature. Both enums are
`#[non_exhaustive]`. HTTP failures are boxed inside `AwsError` rather than exposing the HTTP client's error type, and
`AwsError::is_transient()` reports whether a failed request is worth retrying.

## Features

The APIs in this crate should be configured to require only the dependencies they need, leaving the consumer to include
as much or little as they desire. By default, the `aws` and `model` features are enabled. Decoding of volume headers
and LDM records' NEXRAD messages via `nexrad-decode` is always available. The following named features are helpful
groupings of dependencies/behavior:

- `aws` (default) - Enables accessing archive and real-time NEXRAD data from AWS Open Data, including
  `realtime::chunk_stream`, an async stream that polls for new real-time chunks. WASM-compatible.
- `model` (default) - Provides mappings to a common radar data model, particularly for mapping `volume::File` into a
  `Scan`.
- `parallel` - Parallelize LDM record decompression and decoding using Rayon.

## Minimum Supported Rust Version

Rust 1.88. The MSRV is only raised in minor releases, never in patch releases.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the changes in each release, including the breaking changes in 1.0.0.
To upgrade from 0.x or a release candidate, see [Migrating to 1.0](https://github.com/danielway/nexrad/blob/main/docs/migrating-to-1.0.md).
