//! # nexrad
//!
//! A Rust library for working with NEXRAD (Next Generation Weather Radar) data.
//!
//! This crate serves as the main entry point for the NEXRAD library suite, providing
//! ergonomic top-level functions and re-exports from the underlying crates:
//!
//! - `nexrad-model` - Core data model types (Scan, Sweep, Radial, Site)
//! - `nexrad-decode` - Binary protocol decoding for Archive II format
//! - `nexrad-data` - Data access (local files, AWS S3)
//! - `nexrad-render` - Visualization and rendering
//!
//! ## Quick Start
//!
//! Load radar data with a single function call:
//!
//! ```ignore
//! // Load from a local file
//! let volume = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
//! println!("VCP: {}, {} sweeps",
//!     volume.coverage_pattern_number(),
//!     volume.sweeps().len());
//!
//! // Access data
//! for sweep in volume.sweeps() {
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
//! use chrono::NaiveDate;
//!
//! let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
//! let volume = nexrad::download_latest("KTLX", date).await?;
//! # Ok::<(), nexrad::Error>(())
//! ```
//!
//! ## Terminology
//!
//! The [`prelude`] module provides type aliases that align with standard radar terminology:
//!
//! | Term | Type | Description |
//! |------|------|-------------|
//! | Volume | [`model::data::Scan`] | Complete radar scan at multiple elevations |
//! | Sweep | [`model::data::Sweep`] | Single rotation at one elevation angle |
//! | Radial | [`model::data::Radial`] | Single beam direction with moment data |
//! | Moment | [`model::data::MomentData`] | Per-gate measurements for one data type |
//! | Gate | [`model::data::MomentValue`] | Individual range bin measurement |
//!
//! ```ignore
//! use nexrad::prelude::*;
//!
//! let volume: Volume = nexrad::load_file("radar.ar2v")?;
//! for sweep in volume.sweeps() {
//!     for radial in sweep.radials() {
//!         // Access reflectivity moment data
//!         if let Some(moment) = radial.reflectivity() {
//!             for gate in moment.values() {
//!                 match gate {
//!                     GateValue::Value(dbz) => println!("dBZ: {}", dbz),
//!                     GateValue::BelowThreshold => {},
//!                     _ => {}
//!                 }
//!             }
//!         }
//!     }
//! }
//! # Ok::<(), nexrad::Error>(())
//! ```
//!
//! ## Features
//!
//! All core features are enabled by default. Additional features can be enabled:
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `model` | Core data model types (default) |
//! | `decode` | Protocol decoding (default) |
//! | `data` | Data access (default) |
//! | `render` | Visualization and rendering (default) |
//! | `aws` | Enable AWS S3 downloads ([`download_latest`], [`download_at`], [`list_volumes`]) |
//! | `serde` | Serialization support for model types |
//! | `uom` | Type-safe units of measure |
//! | `chrono` | Date/time type support |
//! | `full` | Enable all features |
//!
//! ## Error Handling
//!
//! This crate provides unified error types via [`Error`] and [`Result<T>`]. All sub-crate
//! errors automatically convert to the unified type via `From` traits, enabling seamless
//! error propagation:
//!
//! ```ignore
//! fn process_volume() -> nexrad::Result<()> {
//!     let data = std::fs::read("volume.ar2")?;  // io::Error converts
//!     let volume = nexrad::data::volume::File::new(data);
//!     let scan = volume.scan()?;  // data/decode/model errors convert
//!     Ok(())
//! }
//! ```
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
//! | `nexrad-decode` | Low-level binary parsing per NOAA ICD 2620010H |
//! | `nexrad-data` | Archive II file handling and AWS S3 access |
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
//!   - ✓ Parsing NEXRAD Level II message format (NOAA ICD 2620010H)
//!   - ✓ Conversion to model types (when feature enabled)
//!   - ✗ No I/O operations (operates on byte slices)
//!   - ✗ No file or network access
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-data`**: File I/O and network access
//!   - ✓ Archive II file handling (including limited volume header decoding)
//!   - ✓ AWS S3 integration (when `aws` feature enabled)
//!   - ✓ Decompression and format handling
//!   - ✓ Uses `nexrad-decode` for message parsing
//!   - ✓ Uses `nexrad-model` for high-level types
//!   - ✗ No rendering or visualization
//!   - ✗ No CLI or user interaction
//!
//! - **`nexrad-render`**: Visualization and image rendering
//!   - ✓ Render radar data to in-memory images
//!   - ✓ Apply color scales to moment data
//!   - ✓ Consume `nexrad-model` types
//!   - ✗ No I/O operations
//!   - ✗ No data access or parsing

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

pub mod prelude;
pub mod result;

pub use result::{Error, Result};

// ============================================================================
// Top-level volume loading functions
// ============================================================================

/// Load a volume from raw Archive II data bytes.
///
/// This function automatically handles decompression of bzip2-compressed LDM records
/// and decodes the NEXRAD messages into the high-level data model.
///
/// # Example
///
/// ```ignore
/// let data = std::fs::read("KTLX20230520_201643_V06.ar2v")?;
/// let volume = nexrad::load(&data)?;
/// println!("VCP: {}, {} sweeps",
///     volume.coverage_pattern_number(),
///     volume.sweeps().len());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the data cannot be parsed as a valid Archive II file,
/// decompression fails, or the messages cannot be decoded.
#[cfg(all(feature = "data", feature = "model"))]
pub fn load(data: &[u8]) -> Result<model::data::Scan> {
    let file = data::volume::File::new(data.to_vec());
    Ok(file.scan()?)
}

/// Load a volume from a file path.
///
/// This is a convenience wrapper around [`load`] that reads the file from disk first.
///
/// # Example
///
/// ```ignore
/// let volume = nexrad::load_file("KTLX20230520_201643_V06.ar2v")?;
/// println!("VCP: {}", volume.coverage_pattern_number());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
#[cfg(all(feature = "data", feature = "model"))]
pub fn load_file<P: AsRef<std::path::Path>>(path: P) -> Result<model::data::Scan> {
    let data = std::fs::read(path)?;
    load(&data)
}

/// Download the most recent volume for a site on a given date.
///
/// Returns the last available archive file for the specified date. This is useful
/// when you want the most complete data available for a particular day.
///
/// # Example
///
/// ```ignore
/// use chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let volume = nexrad::download_latest("KTLX", date).await?;
/// println!("VCP: {}", volume.coverage_pattern_number());
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::NoDataAvailable`] if no archive files exist for the site/date,
/// or a network/parsing error if download or decoding fails.
#[cfg(all(feature = "data", feature = "model", feature = "aws"))]
pub async fn download_latest(site: &str, date: chrono::NaiveDate) -> Result<model::data::Scan> {
    let files = data::aws::archive::list_files(site, &date).await?;
    let file_id = files
        .last()
        .cloned()
        .ok_or_else(|| Error::NoDataAvailable {
            site: site.to_string(),
            date: date.to_string(),
        })?;
    let file = data::aws::archive::download_file(file_id).await?;
    Ok(file.scan()?)
}

/// Download the volume that overlaps a specific datetime.
///
/// Finds the archive file whose collection period contains the requested datetime.
/// Archive files typically span approximately 5 minutes of data collection, so this
/// function returns the volume that was being collected at the specified time.
///
/// # Example
///
/// ```ignore
/// use chrono::NaiveDateTime;
///
/// // Download the volume that was being collected at 20:16:43 UTC
/// let dt = NaiveDateTime::parse_from_str(
///     "2023-05-20 20:16:43",
///     "%Y-%m-%d %H:%M:%S"
/// ).unwrap();
/// let volume = nexrad::download_at("KTLX", dt).await?;
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::NoDataAvailable`] if no archive files exist for the site/date
/// or if no file overlaps the requested time.
#[cfg(all(feature = "data", feature = "model", feature = "aws"))]
pub async fn download_at(site: &str, datetime: chrono::NaiveDateTime) -> Result<model::data::Scan> {
    let files = data::aws::archive::list_files(site, &datetime.date()).await?;

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
            date: datetime.date().to_string(),
        })?;

    let file = data::aws::archive::download_file(file_id).await?;
    Ok(file.scan()?)
}

/// List available volumes for a site and date.
///
/// Returns identifiers for all archive files available on the specified date.
/// These identifiers can be used to selectively download specific volumes.
///
/// # Example
///
/// ```ignore
/// use chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let volumes = nexrad::list_volumes("KTLX", date).await?;
/// println!("Found {} volumes", volumes.len());
/// for vol in &volumes {
///     println!("  {:?}", vol);
/// }
/// # Ok::<(), nexrad::Error>(())
/// ```
#[cfg(all(feature = "data", feature = "aws"))]
pub async fn list_volumes(
    site: &str,
    date: chrono::NaiveDate,
) -> Result<Vec<data::aws::archive::Identifier>> {
    Ok(data::aws::archive::list_files(site, &date).await?)
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
