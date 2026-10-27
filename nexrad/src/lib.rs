//! # nexrad
//!
//! A Rust library for working with NEXRAD (Next Generation Weather Radar) data.
//!
//! This crate serves as the main entry point for the NEXRAD library suite, providing
//! ergonomic top-level functions and re-exports from the underlying crates:
//!
//! - `nexrad-model` as `model` - Core data model types (Scan, Sweep, Radial, Site)
//! - `nexrad-decode` as `decode` - Binary protocol decoding for Archive II format
//! - `nexrad-data` as `data` - Data access (local files, AWS S3)
//! - `nexrad-process` as `process` - Radar data processing algorithms
//! - `nexrad-render` as `render` - Visualization and rendering
//! - `chrono` as `chrono` - Date and time types, re-exported because `download_at` and
//!   `list_scans` take them
//! - `futures_util` as `futures_util` (with `aws`) - Stream utilities, re-exported because the
//!   real-time functions return a `Stream`, whose `StreamExt::next` lives there
//!
//! ## Quick Start
//!
//! Load radar data with a single function call:
//!
//! ```no_run
//! # #[cfg(feature = "data")]
//! # fn run() -> nexrad::Result<()> {
//! // Load from a local file
//! let scan = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
//! println!("{}, {} sweeps",
//!     scan.coverage_pattern_number(),
//!     scan.sweeps().len());
//!
//! // Access data
//! for sweep in scan.sweeps() {
//!     for radial in sweep.radials() {
//!         if let Some(reflectivity) = radial.reflectivity() {
//!             println!("{} gates", reflectivity.gate_count());
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Or download directly from the NEXRAD archive on AWS (requires `aws` feature):
//!
//! ```no_run
//! # #[cfg(feature = "aws")]
//! # async fn run() -> nexrad::Result<()> {
//! let scan = nexrad::download_latest("KTLX").await?;
//! # Ok(())
//! # }
//! ```
//!
//! Real-time data arrives through `stream`, which yields chunks as they are published, and
//! `stream_scans`, which yields one `Scan` per completed volume.
//!
//! Helpers that need only model types are methods on those types: `Sweep::field`,
//! `Scan::fields` and `Scan::first_field` extract `SweepField`s, and
//! `Scan::coordinate_system` builds a `RadarCoordinateSystem` from the scan's site. The
//! static site registry lives in `nexrad::model::meta::registry` (`sites`, `site_by_id`,
//! `nearest_site`).
//!
//! ## Features
//!
//! All features are enabled by default through the `full` feature. Each feature enables the
//! features it depends on, listed under "Implies":
//!
//! | Feature | Description | Implies | Notable dependencies | WASM-compatible |
//! |---------|-------------|---------|----------------------|-----------------|
//! | `model` | Core data model types | | | Yes |
//! | `decode` | Protocol decoding | `model` | zerocopy | Yes |
//! | `data` | Data access for local Archive II files (`load`, `load_file`) | `decode`, `model` | bzip2 | Yes |
//! | `render` | Visualization and rendering | `model` | image | Yes |
//! | `process` | Radar data processing algorithms | `model` | | Yes |
//! | `aws` | AWS S3 downloads (`download`, `download_latest`, `download_at`, `list_scans`), real-time streaming (`stream`, `stream_with`, `stream_scans`, `stream_scans_with`) and the `futures_util` re-export | `data`, `chrono` | reqwest, futures-util | Yes |
//! | `parallel` | Parallel record decompression and decoding | `data` | rayon | No |
//! | `serde` | Serialization support for model types | | serde | Yes |
//! | `uom` | Type-safe units of measure | | uom | Yes |
//! | `chrono` | Date/time types on model types, and the `chrono` re-export | | chrono | Yes |
//! | `wasm` | All WASM-compatible features | `model`, `decode`, `data`, `render`, `process`, `aws`, `serde`, `uom`, `chrono` | | Yes |
//! | `full` | Enable all features (default) | `wasm`, `parallel` | | No |
//!
//! The `aws` feature's TLS stack (rustls with the `aws-lc-rs` crypto provider) compiles C code,
//! which needs a C compiler and, on Windows x86 targets, NASM.
//!
//! ### Common Configurations
//!
//! ```toml
//! # Minimal - local file processing only (no rendering, no networking)
//! nexrad = { version = "1.0", default-features = false, features = ["data"] }
//!
//! # With AWS S3 downloads
//! nexrad = { version = "1.0", default-features = false, features = ["aws"] }
//! tokio = { version = "1", features = ["full"] }
//!
//! # Full feature set
//! nexrad = { version = "1.0", features = ["full"] }
//! ```
//!
//! ## Error Handling
//!
//! This crate provides unified error types via [`Error`] and [`Result<T>`]. All sub-crate
//! errors automatically convert to the unified type via `From` traits, enabling seamless
//! error propagation:
//!
//! ```no_run
//! # #[cfg(feature = "data")]
//! fn process_scan() -> nexrad::Result<()> {
//!     let data = std::fs::read("volume.ar2")?; // io::Error converts
//!     let file = nexrad::data::volume::File::new(data).decompress()?;
//!     let scan = file.scan()?; // data/decode/model errors convert
//!     println!("{} sweeps", scan.sweeps().len());
//!     Ok(())
//! }
//! ```
//!
//! Each variant names the layer an error originated in: decode, model and I/O errors raised
//! inside `nexrad-data` surface as `Error::Decode`, `Error::Model` and `Error::Io`, and only
//! failures specific to data access (decompression, archive layout, AWS) surface as
//! `Error::Data`.
//!
//! See the [`result`] module for detailed error handling documentation.
//!
//! ## Crate Organization
//!
//! For more specialized use cases, you can depend on individual crates directly:
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `nexrad-model` | Domain types with optional serde/chrono/uom support |
//! | `nexrad-decode` | Low-level binary parsing per NOAA ICD 2620002AA |
//! | `nexrad-data` | Archive II file handling and AWS S3 access |
//! | `nexrad-process` | Radar data processing and derived products |
//! | `nexrad-render` | Visualization and image rendering |
//!
//! ## Crate Responsibility Boundaries
//!
//! This facade crate enforces clear separation of concerns across the library suite:
//!
//! ### Re-exported Crates (Part of Public API)
//!
//! - **`nexrad-model`**: Pure data structures and transformations
//!   - ✓ Domain types (Scan, Sweep, Radial, Site)
//!   - ✓ Data transformations and validations
//!   - ✗ No I/O operations (file, network, stdio)
//!   - ✗ No binary parsing or encoding
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-decode`**: Binary protocol parsing
//!   - ✓ Parsing NEXRAD Level II message format (NOAA ICD 2620002AA)
//!   - ✓ All conversion to model types, including `ScanBuilder` (`model` feature)
//!   - ✗ No I/O operations (operates on byte slices)
//!   - ✗ No file or network access
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-data`**: File I/O and network access
//!   - ✓ Archive II file handling (including limited volume header decoding)
//!   - ✓ AWS S3 integration (when `aws` feature enabled)
//!   - ✓ Decompression and format handling
//!   - ✓ Uses `nexrad-decode` for message parsing and for conversion to `nexrad-model` types
//!   - ✗ No protocol-to-model mapping of its own
//!   - ✗ No rendering or visualization
//!   - ✗ No CLI or user interaction
//!
//! - **`nexrad-process`**: Processing algorithms
//!   - ✓ Filters, smoothing and derived products over `nexrad-model` fields
//!   - ✓ Threshold region detection
//!   - ✗ No I/O operations
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-render`**: Visualization and image rendering
//!   - ✓ Render radar data to in-memory images
//!   - ✓ Apply color scales to moment data
//!   - ✓ Consume `nexrad-model` types
//!   - ✗ No I/O operations
//!   - ✗ No data access or parsing
//!
//! ## Lower-Level APIs
//!
//! For advanced use cases, the sub-crates provide finer-grained control.
//!
//! ### Direct File Access with nexrad-data
//!
//! Work directly with Archive II files to access raw LDM records:
//!
//! ```no_run
//! # #[cfg(feature = "data")]
//! # fn run() -> nexrad::Result<()> {
//! use nexrad::data::volume::File;
//!
//! let data = std::fs::read("volume.ar2v")?;
//! let file = File::new(data).decompress()?;
//!
//! // Access raw records before decoding
//! for record in file.records()? {
//!     println!("Record: {} bytes, compressed: {}",
//!         record.data().len(),
//!         record.compressed());
//! }
//!
//! // Or decode directly to a Scan
//! let scan = file.scan()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Binary Message Decoding with nexrad-decode
//!
//! Parse NEXRAD messages from raw (decompressed) bytes:
//!
//! ```no_run
//! # #[cfg(feature = "decode")]
//! # fn run(message_bytes: &[u8]) -> nexrad::decode::Result<()> {
//! use nexrad::decode::messages::{decode_messages, MessageContents};
//!
//! // After extracting decompressed message bytes from an Archive II LDM record
//! let messages = decode_messages(message_bytes)?;
//!
//! for message in &messages {
//!     match message.contents() {
//!         MessageContents::DigitalRadarData(m) => {
//!             let header = m.header();
//!             println!("Site: {:?}", header.radar_identifier());
//!             println!("Azimuth: {} deg", header.azimuth_angle_degrees());
//!             println!("Elevation: {} deg", header.elevation_angle_degrees());
//!         }
//!         MessageContents::RdaVolumeCoveragePattern(v) => {
//!             println!("VCP: {}", v.header().vcp_number());
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Rendering with nexrad-render
//!
//! Create visualizations from radar data:
//!
//! ```no_run
//! # #[cfg(all(feature = "render", feature = "data"))]
//! # fn run() -> nexrad::Result<()> {
//! use nexrad::model::data::Product;
//! use nexrad::render::{default_scale, render_sweep, RenderOptions};
//!
//! let scan = nexrad::load_file("volume.ar2v")?;
//! let sweep = scan.sweeps().first().unwrap();
//! let field = sweep.field(Product::Reflectivity).unwrap();
//!
//! let options = RenderOptions::new(1024, 1024);
//! let color_scale = default_scale(Product::Reflectivity);
//!
//! let result = render_sweep(&field, &color_scale, &options)?;
//! result.save("output.png")?;
//! # Ok(())
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

/// Unified error types for the NEXRAD facade crate.
pub mod result;

pub use result::{Error, Result};

// ============================================================================
// Top-level scan loading functions
// ============================================================================

/// Load a scan from raw Archive II data bytes.
///
/// This function automatically handles decompression of bzip2-compressed LDM records
/// and decodes the NEXRAD messages into the high-level data model.
///
/// # Example
///
/// ```no_run
/// let data = std::fs::read("KTLX20230520_201643_V06.ar2v")?;
/// let scan = nexrad::load(data)?;
/// println!("VCP: {}, {} sweeps",
///     scan.coverage_pattern_number(),
///     scan.sweeps().len());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the data cannot be parsed as a valid Archive II file,
/// decompression fails, or the messages cannot be decoded.
#[cfg(feature = "data")]
pub fn load(data: Vec<u8>) -> Result<model::data::Scan> {
    let file = data::volume::File::new(data).decompress()?;
    Ok(file.scan()?)
}

/// Load a scan from a file path.
///
/// This is a convenience wrapper around [`load`] that reads the file from disk first.
///
/// # Example
///
/// ```no_run
/// let scan = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
/// println!("{}", scan.coverage_pattern_number());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
#[cfg(feature = "data")]
pub fn load_file<P: AsRef<std::path::Path>>(path: P) -> Result<model::data::Scan> {
    load(std::fs::read(path)?)
}

/// Download a specific scan by its archive identifier.
///
/// This function downloads the archive file, handles decompression if needed, and decodes
/// the data into the high-level model. Use [`list_scans`] to obtain identifiers for
/// available scans, then pass the desired one to this function.
///
/// # Example
///
/// ```no_run
/// use nexrad::chrono::NaiveDate;
///
/// # async fn run() -> nexrad::Result<()> {
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let scans = nexrad::list_scans("KTLX", date).await?;
///
/// // Download the first scan of the day
/// if let Some(id) = scans.first() {
///     let scan = nexrad::download(id).await?;
///     println!("{}", scan.coverage_pattern_number());
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the download fails, or if the file cannot be decompressed or decoded.
#[cfg(feature = "aws")]
pub async fn download(identifier: &data::aws::archive::Identifier) -> Result<model::data::Scan> {
    let file = data::aws::archive::download_file(identifier)
        .await?
        .decompress()?;
    Ok(file.scan()?)
}

/// Download the newest archived scan for a site.
///
/// Lists the archive for the current UTC date and, when that date has no volumes yet, for the
/// previous UTC date, then downloads the last volume found.
///
/// To download the last scan of a given date instead, call [`list_scans`] for that date and pass
/// its last identifier to [`download`].
///
/// # Example
///
/// ```no_run
/// # async fn run() -> nexrad::Result<()> {
/// let scan = nexrad::download_latest("KTLX").await?;
/// println!("{}", scan.coverage_pattern_number());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`Error::NoDataAvailable`], carrying the current UTC date, if neither date has an
/// archive volume. Otherwise returns an error if listing or downloading fails, or if the file
/// cannot be decompressed or decoded.
#[cfg(feature = "aws")]
pub async fn download_latest(site: &str) -> Result<model::data::Scan> {
    let today = chrono::Utc::now().date_naive();
    let mut files = data::aws::archive::list_files(site, today).await?;
    if files.is_empty() {
        if let Some(yesterday) = today.pred_opt() {
            files = data::aws::archive::list_files(site, yesterday).await?;
        }
    }

    let file_id = files.last().ok_or_else(|| Error::NoDataAvailable {
        site: site.to_string(),
        date: today,
    })?;
    download(file_id).await
}

/// How long after its start time an archive volume is taken to overlap a requested datetime in
/// [`download_at`]. The longest routine volume coverage patterns (VCP 31 and 32) take about ten
/// minutes.
#[cfg(feature = "aws")]
const MAX_VOLUME_DURATION: chrono::TimeDelta = chrono::TimeDelta::minutes(15);

/// Download the scan that overlaps a specific datetime.
///
/// Finds the archive volume whose collection period contains the requested UTC datetime: the
/// last volume that started at or before it, which is on the previous UTC date when the datetime
/// precedes the date's first volume. A volume overlaps the datetime when it started at most 15
/// minutes before it.
///
/// # Example
///
/// ```no_run
/// use nexrad::chrono::NaiveDateTime;
///
/// # async fn run() -> nexrad::Result<()> {
/// // Download the scan that was being collected at 20:16:43 UTC
/// let dt = NaiveDateTime::parse_from_str(
///     "2023-05-20 20:16:43",
///     "%Y-%m-%d %H:%M:%S"
/// ).unwrap();
/// let scan = nexrad::download_at("KTLX", dt).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`Error::NoScanAt`] if no archive volume overlaps the requested datetime. Otherwise
/// returns an error if listing or downloading fails, or if the file cannot be decompressed or
/// decoded.
#[cfg(feature = "aws")]
pub async fn download_at(site: &str, datetime: chrono::NaiveDateTime) -> Result<model::data::Scan> {
    let date = datetime.date();
    let day = data::aws::archive::list_files(site, date).await?;

    let needs_previous_day = last_started_by(&day, datetime).is_none();
    let previous_day = match date.pred_opt() {
        Some(previous) if needs_previous_day => {
            data::aws::archive::list_files(site, previous).await?
        }
        _ => Vec::new(),
    };

    let file_id = scan_at(&day, &previous_day, datetime).ok_or_else(|| Error::NoScanAt {
        site: site.to_string(),
        datetime,
    })?;
    download(file_id).await
}

/// Returns the last of `files`, which are in chronological order, that started at or before
/// `datetime`.
#[cfg(feature = "aws")]
fn last_started_by(
    files: &[data::aws::archive::Identifier],
    datetime: chrono::NaiveDateTime,
) -> Option<&data::aws::archive::Identifier> {
    files
        .iter()
        .rfind(|file| file.date_time().naive_utc() <= datetime)
}

/// Selects the volume overlapping `datetime` from the volumes of its date and of the previous
/// date: the last one that started at or before `datetime`, provided it started no more than
/// [`MAX_VOLUME_DURATION`] earlier.
#[cfg(feature = "aws")]
fn scan_at<'a>(
    day: &'a [data::aws::archive::Identifier],
    previous_day: &'a [data::aws::archive::Identifier],
    datetime: chrono::NaiveDateTime,
) -> Option<&'a data::aws::archive::Identifier> {
    last_started_by(day, datetime)
        .or_else(|| last_started_by(previous_day, datetime))
        .filter(|file| datetime - file.date_time().naive_utc() <= MAX_VOLUME_DURATION)
}

/// List available scans for a site and date.
///
/// Returns identifiers for all archive volume files available on the specified UTC date, in
/// chronological order; metadata objects are not included.
/// These identifiers can be used to selectively download specific scans.
///
/// # Example
///
/// ```no_run
/// use nexrad::chrono::NaiveDate;
///
/// # async fn run() -> nexrad::Result<()> {
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let scans = nexrad::list_scans("KTLX", date).await?;
/// println!("Found {} scans", scans.len());
/// for id in &scans {
///     println!("  {id}");
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the archive listing request fails or its response cannot be parsed. A
/// date without data is not an error; it yields an empty list.
#[cfg(feature = "aws")]
pub async fn list_scans(
    site: &str,
    date: chrono::NaiveDate,
) -> Result<Vec<data::aws::archive::Identifier>> {
    Ok(data::aws::archive::list_files(site, date).await?)
}

// ============================================================================
// Real-time data access
// ============================================================================

/// Stream real-time radar data chunks for a site.
///
/// Returns a [`Stream`](futures_util::stream::Stream) that yields chunks as they become available
/// from NOAA's real-time NEXRAD data feed. This is a convenience wrapper around
/// [`data::aws::realtime::chunk_stream`] with default retry policies; [`stream_with`] takes a
/// custom configuration.
///
/// The stream is runtime-agnostic: `sleep` is called with each duration the
/// stream needs to wait and must return a future that completes after that
/// duration. With tokio, pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a
/// timer crate such as `gloo-timers` provides an equivalent sleep future. The
/// stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```no_run
/// use nexrad::futures_util::StreamExt;
///
/// # async fn run() -> nexrad::Result<()> {
/// let stream = nexrad::stream("KTLX", |d| tokio::time::sleep(d));
/// let mut stream = std::pin::pin!(stream);
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     println!("Received chunk: {}", chunk.identifier.name());
/// }
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "aws")]
pub fn stream<S, F>(
    site: &str,
    sleep: S,
) -> impl futures_util::stream::Stream<Item = Result<data::aws::realtime::DownloadedChunk>>
where
    S: Fn(std::time::Duration) -> F,
    F: std::future::Future<Output = ()>,
{
    stream_with(data::aws::realtime::PollConfig::new(site), sleep)
}

/// Stream real-time radar data chunks with a custom configuration.
///
/// Like [`stream`], but takes a [`PollConfig`](data::aws::realtime::PollConfig), which names the
/// site and sets the download and discovery retry policies. This is a wrapper around
/// [`data::aws::realtime::chunk_stream`].
///
/// The stream is runtime-agnostic: `sleep` is called with each duration the
/// stream needs to wait and must return a future that completes after that
/// duration. With tokio, pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a
/// timer crate such as `gloo-timers` provides an equivalent sleep future. The
/// stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```no_run
/// use nexrad::data::aws::realtime::{PollConfig, RetryPolicy};
/// use nexrad::futures_util::StreamExt;
/// use std::time::Duration;
///
/// # async fn run() -> nexrad::Result<()> {
/// let config = PollConfig::new("KTLX").with_download_policy(
///     RetryPolicy::default_download().with_initial_delay(Duration::from_millis(500)),
/// );
/// let stream = nexrad::stream_with(config, |d| tokio::time::sleep(d));
/// let mut stream = std::pin::pin!(stream);
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     println!("Received chunk: {}", chunk.identifier.name());
/// }
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "aws")]
pub fn stream_with<S, F>(
    config: data::aws::realtime::PollConfig,
    sleep: S,
) -> impl futures_util::stream::Stream<Item = Result<data::aws::realtime::DownloadedChunk>>
where
    S: Fn(std::time::Duration) -> F,
    F: std::future::Future<Output = ()>,
{
    use futures_util::StreamExt;
    data::aws::realtime::chunk_stream(config, sleep).map(|r| r.map_err(Error::from))
}

/// Stream complete real-time radar scans for a site.
///
/// Returns a [`Stream`](futures_util::stream::Stream) that yields one
/// [`Scan`](model::data::Scan) per volume from NOAA's real-time NEXRAD data feed. It receives
/// chunks as [`stream`] does, with default retry policies, and assembles each volume with
/// [`data::aws::realtime::assemble_volume`] once its End chunk arrives; [`stream_scans_with`]
/// takes a custom configuration.
///
/// Chunks received before the first Start chunk are skipped, so the volume in progress when the
/// stream starts is not yielded. An error from the chunk stream is yielded as an error item and
/// the partial volume is discarded; the next Start chunk begins a new one. Only complete volumes
/// are yielded.
///
/// The stream is runtime-agnostic: `sleep` is called with each duration the
/// stream needs to wait and must return a future that completes after that
/// duration. With tokio, pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a
/// timer crate such as `gloo-timers` provides an equivalent sleep future. The
/// stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```no_run
/// use nexrad::futures_util::StreamExt;
///
/// # async fn run() -> nexrad::Result<()> {
/// let scans = nexrad::stream_scans("KTLX", |d| tokio::time::sleep(d));
/// let mut scans = std::pin::pin!(scans);
/// while let Some(scan) = scans.next().await {
///     let scan = scan?;
///     println!("{}, {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());
/// }
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "aws")]
pub fn stream_scans<S, F>(
    site: &str,
    sleep: S,
) -> impl futures_util::stream::Stream<Item = Result<model::data::Scan>>
where
    S: Fn(std::time::Duration) -> F,
    F: std::future::Future<Output = ()>,
{
    stream_scans_with(data::aws::realtime::PollConfig::new(site), sleep)
}

/// Stream complete real-time radar scans with a custom configuration.
///
/// Like [`stream_scans`], but takes a [`PollConfig`](data::aws::realtime::PollConfig), which
/// names the site and sets the download and discovery retry policies.
///
/// Chunks received before the first Start chunk are skipped, so the volume in progress when the
/// stream starts is not yielded. An error from the chunk stream is yielded as an error item and
/// the partial volume is discarded; the next Start chunk begins a new one. Only complete volumes
/// are yielded.
///
/// The stream is runtime-agnostic: `sleep` is called with each duration the
/// stream needs to wait and must return a future that completes after that
/// duration. With tokio, pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a
/// timer crate such as `gloo-timers` provides an equivalent sleep future. The
/// stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```no_run
/// use nexrad::data::aws::realtime::PollConfig;
/// use nexrad::futures_util::StreamExt;
///
/// # async fn run() -> nexrad::Result<()> {
/// let scans = nexrad::stream_scans_with(PollConfig::new("KTLX"), |d| tokio::time::sleep(d));
/// let mut scans = std::pin::pin!(scans);
/// while let Some(scan) = scans.next().await {
///     let scan = scan?;
///     println!("{}, {} sweeps", scan.coverage_pattern_number(), scan.sweeps().len());
/// }
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "aws")]
pub fn stream_scans_with<S, F>(
    config: data::aws::realtime::PollConfig,
    sleep: S,
) -> impl futures_util::stream::Stream<Item = Result<model::data::Scan>>
where
    S: Fn(std::time::Duration) -> F,
    F: std::future::Future<Output = ()>,
{
    use futures_util::StreamExt;
    let mut volume = VolumeAccumulator::default();
    stream_with(config, sleep).filter_map(move |item| {
        let scan = volume.accept(item).map(|chunks| {
            chunks.and_then(|chunks| Ok(data::aws::realtime::assemble_volume(chunks)?))
        });
        futures_util::future::ready(scan)
    })
}

/// Collects the chunks of one real-time volume, from its Start chunk through its End chunk.
#[cfg(feature = "aws")]
#[derive(Default)]
struct VolumeAccumulator {
    /// The chunks of the volume in progress, in sequence order.
    chunks: Vec<data::aws::realtime::Chunk>,
    /// The scan time and sequence number of the last chunk added to `chunks`, or `None` when no
    /// volume is in progress.
    last: Option<(chrono::NaiveDateTime, usize)>,
}

#[cfg(feature = "aws")]
impl VolumeAccumulator {
    /// Takes the next item from the chunk stream. Returns the volume's chunks once its End chunk
    /// completes it, the error when the item is an error, and `None` otherwise.
    ///
    /// A Start chunk begins a new volume, replacing any partial one. Any other chunk is added
    /// only when it directly follows the last chunk added, in the same volume; otherwise the
    /// partial volume is discarded, as it is on an error.
    fn accept(
        &mut self,
        item: Result<data::aws::realtime::DownloadedChunk>,
    ) -> Option<Result<Vec<data::aws::realtime::Chunk>>> {
        use data::aws::realtime::ChunkType;

        let downloaded = match item {
            Ok(downloaded) => downloaded,
            Err(error) => {
                self.discard();
                return Some(Err(error));
            }
        };

        let identifier = &downloaded.identifier;
        let chunk_type = identifier.chunk_type();
        let position = (*identifier.date_time_prefix(), identifier.sequence());
        if chunk_type == ChunkType::Start {
            self.chunks.clear();
        } else {
            let follows = self
                .last
                .is_some_and(|(scan_time, sequence)| (scan_time, sequence + 1) == position);
            if !follows {
                self.discard();
                return None;
            }
        }

        self.chunks.push(downloaded.chunk);
        if chunk_type == ChunkType::End {
            self.last = None;
            Some(Ok(std::mem::take(&mut self.chunks)))
        } else {
            self.last = Some(position);
            None
        }
    }

    /// Drops the volume in progress.
    fn discard(&mut self) {
        self.chunks.clear();
        self.last = None;
    }
}

// ============================================================================
// Sub-crate re-exports
// ============================================================================

/// Re-export of `nexrad-model` for core data types.
#[cfg(feature = "model")]
pub use nexrad_model as model;

/// Re-export of `nexrad-decode` for binary protocol decoding.
#[cfg(feature = "decode")]
pub use nexrad_decode as decode;

/// Re-export of `nexrad-data` for data access and AWS integration.
#[cfg(feature = "data")]
pub use nexrad_data as data;

/// Re-export of `nexrad-render` for visualization and rendering.
#[cfg(feature = "render")]
pub use nexrad_render as render;

/// Re-export of `nexrad-process` for radar data processing algorithms.
#[cfg(feature = "process")]
pub use nexrad_process as process;

/// Re-export of `chrono`, because `download_at` and `list_scans` take chrono types.
#[cfg(feature = "chrono")]
pub use chrono;

/// Re-export of `futures_util`, because the real-time functions return a `Stream`, whose
/// `StreamExt::next` lives there.
#[cfg(feature = "aws")]
pub use futures_util;

#[cfg(all(test, feature = "aws"))]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use data::aws::archive::Identifier;
    use data::aws::realtime::{Chunk, ChunkIdentifier, ChunkType, DownloadedChunk, VolumeIndex};

    fn datetime(value: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").expect("valid datetime")
    }

    fn identifiers(names: &[&str]) -> Vec<Identifier> {
        names
            .iter()
            .map(|name| Identifier::parse(name).expect("valid identifier"))
            .collect()
    }

    fn selected<'a>(
        day: &'a [Identifier],
        previous_day: &'a [Identifier],
        at: &str,
    ) -> Option<&'a str> {
        scan_at(day, previous_day, datetime(at)).map(Identifier::name)
    }

    #[test]
    fn scan_at_selects_the_last_volume_started_by_the_datetime() {
        let day = identifiers(&[
            "KTLX20230520_200712_V06",
            "KTLX20230520_201643_V06",
            "KTLX20230520_202612_V06",
        ]);

        assert_eq!(
            selected(&day, &[], "2023-05-20 20:20:00"),
            Some("KTLX20230520_201643_V06")
        );
        assert_eq!(
            selected(&day, &[], "2023-05-20 20:16:43"),
            Some("KTLX20230520_201643_V06")
        );
    }

    #[test]
    fn scan_at_crosses_midnight() {
        let previous_day = identifiers(&["KTLX20230519_234512_V06", "KTLX20230519_235502_V06"]);
        let day = identifiers(&["KTLX20230520_000452_V06"]);

        assert_eq!(
            selected(&day, &previous_day, "2023-05-20 00:01:00"),
            Some("KTLX20230519_235502_V06")
        );
        assert_eq!(
            selected(&[], &previous_day, "2023-05-20 00:01:00"),
            Some("KTLX20230519_235502_V06")
        );
    }

    #[test]
    fn scan_at_rejects_a_volume_started_too_long_before() {
        let day = identifiers(&["KTLX20230520_200000_V06"]);

        assert_eq!(
            selected(&day, &[], "2023-05-20 20:15:00"),
            Some("KTLX20230520_200000_V06")
        );
        assert_eq!(selected(&day, &[], "2023-05-20 20:15:01"), None);

        let previous_day = identifiers(&["KTLX20230519_230000_V06"]);
        assert_eq!(selected(&[], &previous_day, "2023-05-20 00:01:00"), None);
    }

    #[test]
    fn scan_at_finds_nothing_before_the_first_volume() {
        let day = identifiers(&["KTLX20230520_000452_V06"]);

        assert_eq!(selected(&day, &[], "2023-05-20 00:01:00"), None);
        assert_eq!(selected(&[], &[], "2023-05-20 00:01:00"), None);
    }

    fn chunk(scan_time: &str, sequence: usize, chunk_type: ChunkType) -> Result<DownloadedChunk> {
        let identifier = ChunkIdentifier::new(
            "KTLX",
            VolumeIndex::new(1).expect("valid volume index"),
            datetime(scan_time),
            sequence,
            chunk_type,
            None,
        );
        let data = match chunk_type {
            ChunkType::Start => b"AR2V0006.".to_vec(),
            _ => b"\0\0\0\0BZh9".to_vec(),
        };
        let chunk = Chunk::new(data).expect("recognized chunk format");
        Ok(DownloadedChunk::new(identifier, chunk, 1))
    }

    fn error() -> Result<DownloadedChunk> {
        Err(Error::Io(std::io::Error::other("connection reset")))
    }

    /// Feeds `items` to a new accumulator and returns, per item, `None` when nothing was
    /// yielded, `Some(Ok(n))` for a volume of `n` chunks and `Some(Err(()))` for an error.
    fn accept_all(
        items: impl IntoIterator<Item = Result<DownloadedChunk>>,
    ) -> Vec<Option<std::result::Result<usize, ()>>> {
        let mut volume = VolumeAccumulator::default();
        items
            .into_iter()
            .map(|item| {
                volume
                    .accept(item)
                    .map(|chunks| chunks.map(|chunks| chunks.len()).map_err(|_| ()))
            })
            .collect()
    }

    const VOLUME: &str = "2023-05-20 20:16:43";
    const NEXT_VOLUME: &str = "2023-05-20 20:21:30";

    #[test]
    fn accumulator_yields_a_complete_volume() {
        let results = accept_all([
            chunk(VOLUME, 1, ChunkType::Start),
            chunk(VOLUME, 2, ChunkType::Intermediate),
            chunk(VOLUME, 3, ChunkType::End),
        ]);

        assert_eq!(results, [None, None, Some(Ok(3))]);
    }

    #[test]
    fn accumulator_skips_chunks_before_the_first_start() {
        let results = accept_all([
            chunk(VOLUME, 5, ChunkType::Intermediate),
            chunk(VOLUME, 6, ChunkType::End),
            chunk(NEXT_VOLUME, 1, ChunkType::Start),
            chunk(NEXT_VOLUME, 2, ChunkType::End),
        ]);

        assert_eq!(results, [None, None, None, Some(Ok(2))]);
    }

    #[test]
    fn accumulator_discards_a_volume_missing_a_chunk() {
        // The chunk stream joins mid-volume by yielding the Start chunk and then the latest one.
        let results = accept_all([
            chunk(VOLUME, 1, ChunkType::Start),
            chunk(VOLUME, 7, ChunkType::Intermediate),
            chunk(VOLUME, 8, ChunkType::End),
            chunk(NEXT_VOLUME, 1, ChunkType::Start),
            chunk(NEXT_VOLUME, 2, ChunkType::Intermediate),
            chunk(NEXT_VOLUME, 3, ChunkType::End),
        ]);

        assert_eq!(results, [None, None, None, None, None, Some(Ok(3))]);
    }

    #[test]
    fn accumulator_forwards_errors_and_discards_the_partial_volume() {
        let results = accept_all([
            chunk(VOLUME, 1, ChunkType::Start),
            chunk(VOLUME, 2, ChunkType::Intermediate),
            error(),
            chunk(VOLUME, 3, ChunkType::End),
            chunk(NEXT_VOLUME, 1, ChunkType::Start),
            chunk(NEXT_VOLUME, 2, ChunkType::End),
        ]);

        assert_eq!(
            results,
            [None, None, Some(Err(())), None, None, Some(Ok(2))]
        );
    }

    #[test]
    fn accumulator_restarts_on_a_new_start_chunk() {
        let results = accept_all([
            chunk(VOLUME, 1, ChunkType::Start),
            chunk(VOLUME, 2, ChunkType::Intermediate),
            chunk(NEXT_VOLUME, 1, ChunkType::Start),
            chunk(NEXT_VOLUME, 2, ChunkType::Intermediate),
            chunk(NEXT_VOLUME, 3, ChunkType::End),
        ]);

        assert_eq!(results, [None, None, None, None, Some(Ok(3))]);
    }

    #[test]
    fn streams_are_send_with_a_send_sleep() {
        fn assert_send<T: Send>(_: &T) {}

        let sleep = |_: std::time::Duration| async {};
        assert_send(&stream("KTLX", sleep));
        assert_send(&stream_scans("KTLX", sleep));
    }

    #[test]
    fn accumulator_rejects_a_chunk_from_another_volume() {
        let results = accept_all([
            chunk(VOLUME, 1, ChunkType::Start),
            chunk(NEXT_VOLUME, 2, ChunkType::End),
        ]);

        assert_eq!(results, [None, None]);
    }
}
