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
//! - `chrono` as `chrono` - Date and time types, re-exported because `download_latest`,
//!   `download_at` and `list_scans` take them
//!
//! ## Quick Start
//!
//! Load radar data with a single function call:
//!
//! ```ignore
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
//! # Ok::<(), nexrad::Error>(())
//! ```
//!
//! Or download directly from the NEXRAD archive on AWS (requires `aws` feature):
//!
//! ```ignore
//! use nexrad::chrono::NaiveDate;
//!
//! let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
//! let scan = nexrad::download_latest("KTLX", date).await?;
//! # Ok::<(), nexrad::Error>(())
//! ```
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
//! | `aws` | AWS S3 downloads (`download`, `download_latest`, `download_at`, `list_scans`) and real-time streaming (`stream`) | `data`, `chrono` | reqwest, futures-util | Yes |
//! | `parallel` | Parallel record decompression and decoding | `data` | rayon | No |
//! | `serde` | Serialization support for model types | | serde | Yes |
//! | `uom` | Type-safe units of measure | | uom | Yes |
//! | `chrono` | Date/time types on model types, and the `chrono` re-export | | chrono | Yes |
//! | `wasm` | All WASM-compatible features | `model`, `decode`, `data`, `render`, `process`, `aws`, `serde`, `uom`, `chrono` | | Yes |
//! | `full` | Enable all features (default) | `wasm`, `parallel` | | No |
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
//!   - ✓ Storm cell detection
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
//! ```ignore
//! use nexrad::data::volume::File;
//!
//! let data = std::fs::read("volume.ar2v")?;
//! let file = File::new(data);
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
//! # Ok::<(), nexrad::Error>(())
//! ```
//!
//! ### Binary Message Decoding with nexrad-decode
//!
//! Parse NEXRAD messages from raw (decompressed) bytes:
//!
//! ```ignore
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
//! # Ok::<(), nexrad::decode::Error>(())
//! ```
//!
//! ### Rendering with nexrad-render
//!
//! Create visualizations from radar data:
//!
//! ```ignore
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
//! # Ok::<(), nexrad::Error>(())
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]

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
/// ```ignore
/// let data = std::fs::read("KTLX20230520_201643_V06.ar2v")?;
/// let scan = nexrad::load(&data)?;
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
pub fn load(data: &[u8]) -> Result<model::data::Scan> {
    let file = data::volume::File::new(data.to_vec()).decompress()?;
    Ok(file.scan()?)
}

/// Load a scan from a file path.
///
/// This is a convenience wrapper around [`load`] that reads the file from disk first.
///
/// # Example
///
/// ```ignore
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
    let data = std::fs::read(path)?;
    load(&data)
}

/// Download a specific scan by its archive identifier.
///
/// This function downloads the archive file, handles decompression if needed, and decodes
/// the data into the high-level model. Use [`list_scans`] to obtain identifiers for
/// available scans, then pass the desired one to this function.
///
/// # Example
///
/// ```ignore
/// use nexrad::chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let scans = nexrad::list_scans("KTLX", date).await?;
///
/// // Download the first scan of the day
/// if let Some(id) = scans.first() {
///     let scan = nexrad::download(id).await?;
///     println!("{}", scan.coverage_pattern_number());
/// }
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the download, decompression, or decoding fails.
#[cfg(feature = "aws")]
pub async fn download(identifier: &data::aws::archive::Identifier) -> Result<model::data::Scan> {
    let file = data::aws::archive::download_file(identifier)
        .await?
        .decompress()?;
    Ok(file.scan()?)
}

/// Download the most recent scan for a site on a given date.
///
/// Returns the last available archive file for the specified date. This is useful
/// when you want the most complete data available for a particular day.
///
/// # Example
///
/// ```ignore
/// use nexrad::chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let scan = nexrad::download_latest("KTLX", date).await?;
/// println!("{}", scan.coverage_pattern_number());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::NoDataAvailable`] if no archive files exist for the site/date,
/// or a network/parsing error if download or decoding fails.
#[cfg(feature = "aws")]
pub async fn download_latest(site: &str, date: chrono::NaiveDate) -> Result<model::data::Scan> {
    let files = data::aws::archive::list_files(site, date).await?;
    let file_id = files
        .last()
        .cloned()
        .ok_or_else(|| Error::NoDataAvailable {
            site: site.to_string(),
            date,
        })?;
    download(&file_id).await
}

/// Download the scan that overlaps a specific datetime.
///
/// Finds the archive file whose collection period contains the requested datetime.
/// Archive files typically span approximately 5 minutes of data collection, so this
/// function returns the scan that was being collected at the specified time.
///
/// # Example
///
/// ```ignore
/// use nexrad::chrono::NaiveDateTime;
///
/// // Download the scan that was being collected at 20:16:43 UTC
/// let dt = NaiveDateTime::parse_from_str(
///     "2023-05-20 20:16:43",
///     "%Y-%m-%d %H:%M:%S"
/// ).unwrap();
/// let scan = nexrad::download_at("KTLX", dt).await?;
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::NoDataAvailable`] if no archive files exist for the site/date
/// or if no file overlaps the requested time.
#[cfg(feature = "aws")]
pub async fn download_at(site: &str, datetime: chrono::NaiveDateTime) -> Result<model::data::Scan> {
    let files = data::aws::archive::list_files(site, datetime.date()).await?;

    // Find the archive file that would contain data at the requested time.
    // Files are sorted chronologically; find the last one with start_time <= requested time.
    let file_id = files
        .iter()
        .rfind(|f| {
            f.date_time()
                .map(|dt| dt.time() <= datetime.time())
                .unwrap_or(false)
        })
        .cloned()
        .ok_or_else(|| Error::NoDataAvailable {
            site: site.to_string(),
            date: datetime.date(),
        })?;

    download(&file_id).await
}

/// List available scans for a site and date.
///
/// Returns identifiers for all archive files available on the specified date.
/// These identifiers can be used to selectively download specific scans.
///
/// # Example
///
/// ```ignore
/// use nexrad::chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let scans = nexrad::list_scans("KTLX", date).await?;
/// println!("Found {} scans", scans.len());
/// for id in &scans {
///     println!("  {:?}", id);
/// }
/// # Ok::<(), nexrad::Error>(())
/// ```
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
/// [`data::aws::realtime::chunk_stream`] with default retry policies.
///
/// The stream is runtime-agnostic: `sleep` is called with each duration the
/// stream needs to wait and must return a future that completes after that
/// duration. With tokio, pass `|d| tokio::time::sleep(d)`. On `wasm32` targets, a
/// timer crate such as `gloo-timers` provides an equivalent sleep future. The
/// stream is `Send` whenever `S` and `F` are `Send`.
///
/// # Example
///
/// ```ignore
/// use futures_util::StreamExt;
///
/// let stream = nexrad::stream("KTLX", |d| tokio::time::sleep(d));
/// let mut stream = std::pin::pin!(stream);
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     println!("Received chunk: {:?}", chunk.identifier);
/// }
/// # Ok::<(), nexrad::Error>(())
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
    use futures_util::StreamExt;
    let config = data::aws::realtime::PollConfig::new(site);
    data::aws::realtime::chunk_stream(config, sleep).map(|r| r.map_err(Error::from))
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

/// Re-export of `chrono`, because `download_latest`, `download_at` and `list_scans` take
/// chrono types.
#[cfg(feature = "chrono")]
pub use chrono;
