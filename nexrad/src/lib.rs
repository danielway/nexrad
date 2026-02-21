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
//! println!("{}, {} sweeps",
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
//! ## Features
//!
//! All core features are enabled by default. Additional features can be enabled:
//!
//! | Feature | Description | Dependencies | WASM-compatible |
//! |---------|-------------|--------------|-----------------|
//! | `model` | Core data model types (default) | Pure Rust | Yes |
//! | `decode` | Protocol decoding (default) | chrono, zerocopy | Yes |
//! | `data` | Data access (default) | bzip2 | Yes |
//! | `render` | Visualization and rendering (default) | image | Yes |
//! | `aws` | Enable AWS S3 downloads (`download_latest`, `download_at`, `list_volumes`) | reqwest | Yes |
//! | `aws-polling` | Real-time polling (`poll_chunks`) | reqwest, tokio | No |
//! | `serde` | Serialization support for model types | serde | Yes |
//! | `uom` | Type-safe units of measure | uom | Yes |
//! | `chrono` | Date/time type support | chrono | Yes |
//! | `full` | Enable all features | All above | No |
//!
//! ### Common Configurations
//!
//! ```toml
//! # Minimal - local file processing only (no rendering)
//! nexrad = { version = "1.0", default-features = false, features = ["model", "decode", "data"] }
//!
//! # With AWS S3 downloads
//! nexrad = { version = "1.0", features = ["aws"] }
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
//! | `nexrad-decode` | Low-level binary parsing per NOAA ICD 2620002AA |
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
//!   - ✓ Parsing NEXRAD Level II message format (NOAA ICD 2620002AA)
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
//!         record.is_compressed());
//! }
//!
//! // Or decode directly to a Scan
//! let scan = file.scan()?;
//! # Ok::<(), nexrad::Error>(())
//! ```
//!
//! ### Binary Message Decoding with nexrad-decode
//!
//! Parse individual NEXRAD messages from raw bytes:
//!
//! ```ignore
//! use nexrad::decode::messages::{decode_message, Message};
//!
//! // After extracting message bytes from an Archive II file
//! let (remaining, message) = decode_message(message_bytes)?;
//!
//! match message {
//!     Message::DigitalRadarData(msg31) => {
//!         println!("Azimuth: {} deg", msg31.azimuth_angle());
//!         println!("Elevation: {} deg", msg31.elevation_angle());
//!     }
//!     Message::VolumeData(msg2) => {
//!         println!("Site: {}", msg2.site_id());
//!     }
//!     _ => {}
//! }
//! # Ok::<(), nexrad::decode::result::Error>(())
//! ```
//!
//! ### Rendering with nexrad-render
//!
//! Create visualizations from radar data:
//!
//! ```ignore
//! use nexrad::render::{render_radials, get_nws_reflectivity_scale, Product, RenderOptions};
//!
//! let volume = nexrad::load_file("volume.ar2v")?;
//! let sweep = volume.sweeps().first().unwrap();
//!
//! let options = RenderOptions::new(1024, 1024);
//! let color_scale = get_nws_reflectivity_scale();
//!
//! let image = render_radials(
//!     sweep.radials(),
//!     Product::Reflectivity,
//!     &color_scale,
//!     &options,
//! )?;
//!
//! image.save("output.png").unwrap();
//! # Ok::<(), nexrad::Error>(())
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![deny(missing_docs)]

/// Unified error types for the NEXRAD facade crate.
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
    let file = data::volume::File::new(data.to_vec()).decompress()?;
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
/// println!("{}", volume.coverage_pattern_number());
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

/// Download a specific volume by its archive identifier.
///
/// This function downloads the volume file, handles decompression if needed, and decodes
/// the data into the high-level model. Use [`list_volumes`] to obtain identifiers for
/// available volumes, then pass the desired one to this function.
///
/// # Example
///
/// ```ignore
/// use chrono::NaiveDate;
///
/// let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
/// let volumes = nexrad::list_volumes("KTLX", date).await?;
///
/// // Download the first volume of the day
/// if let Some(id) = volumes.into_iter().next() {
///     let volume = nexrad::download(id).await?;
///     println!("{}", volume.coverage_pattern_number());
/// }
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the download, decompression, or decoding fails.
#[cfg(all(feature = "data", feature = "model", feature = "aws"))]
pub async fn download(identifier: data::aws::archive::Identifier) -> Result<model::data::Scan> {
    let file = data::aws::archive::download_file(identifier)
        .await?
        .decompress()?;
    Ok(file.scan()?)
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
/// println!("{}", volume.coverage_pattern_number());
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
    download(file_id).await
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

    download(file_id).await
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
// Top-level rendering functions
// ============================================================================

/// Render a sweep's radar data to an RGBA image using the default color scale.
///
/// This is a convenience function that renders with the default color scale for the
/// given product and a 1024x1024 transparent background. For more control over rendering
/// options and color scales, use [`render::render_radials`] directly.
///
/// # Example
///
/// ```ignore
/// use nexrad::render::Product;
///
/// let volume = nexrad::load_file("volume.ar2v")?;
/// let sweep = volume.sweeps().first().unwrap();
/// let image = nexrad::render_sweep(sweep, Product::Reflectivity)?;
/// image.save("output.png").unwrap();
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the sweep has no radials or the requested product is not present.
#[cfg(all(feature = "render", feature = "model"))]
pub fn render_sweep(
    sweep: &model::data::Sweep,
    product: render::Product,
) -> Result<render::RgbaImage> {
    let options = render::RenderOptions::new(1024, 1024).transparent();
    Ok(render::render_radials_default(
        sweep.radials(),
        product,
        &options,
    )?)
}

/// Render a sweep's radar data to an RGBA image with custom options.
///
/// Uses the default color scale for the given product but allows custom render options
/// (image size, background color). For full control over color scales, use
/// [`render::render_radials`] directly.
///
/// # Example
///
/// ```ignore
/// use nexrad::render::{Product, RenderOptions};
///
/// let volume = nexrad::load_file("volume.ar2v")?;
/// let sweep = volume.sweeps().first().unwrap();
/// let options = RenderOptions::new(800, 800).with_background([0, 0, 0, 255]);
/// let image = nexrad::render_sweep_with_options(sweep, Product::Velocity, &options)?;
/// image.save("velocity.png").unwrap();
/// # Ok::<(), nexrad::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the sweep has no radials or the requested product is not present.
#[cfg(all(feature = "render", feature = "model"))]
pub fn render_sweep_with_options(
    sweep: &model::data::Sweep,
    product: render::Product,
    options: &render::RenderOptions,
) -> Result<render::RgbaImage> {
    Ok(render::render_radials_default(
        sweep.radials(),
        product,
        options,
    )?)
}

// ============================================================================
// Real-time data access
// ============================================================================

/// Stream real-time radar data chunks for a site.
///
/// Returns a [`futures::Stream`] that yields chunks as they become available from
/// NOAA's real-time NEXRAD data feed. This is a convenience wrapper around
/// [`data::aws::realtime::chunk_stream`] with default retry policies.
///
/// # Example
///
/// ```ignore
/// use futures::StreamExt;
///
/// let mut stream = nexrad::stream("KTLX");
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     println!("Received chunk: {:?}", chunk.identifier);
/// }
/// # Ok::<(), nexrad::Error>(())
/// ```
#[cfg(all(feature = "data", feature = "aws-polling"))]
pub fn stream(
    site: &str,
) -> impl futures::Stream<Item = Result<data::aws::realtime::DownloadedChunk>> {
    use futures::StreamExt;
    let config = data::aws::realtime::PollConfig::new(site);
    data::aws::realtime::chunk_stream(config).map(|r| r.map_err(Error::Data))
}

// ============================================================================
// Site registry convenience functions
// ============================================================================

/// Returns all NEXRAD radar sites in the static registry.
///
/// This provides a compile-time list of all operational NEXRAD WSR-88D radar sites.
///
/// # Example
///
/// ```
/// for site in nexrad::sites() {
///     println!("{}: {}, {}", site.id, site.city, site.state);
/// }
/// ```
#[cfg(feature = "model")]
pub fn sites() -> &'static [model::meta::registry::SiteEntry] {
    model::meta::registry::sites()
}

/// Look up a NEXRAD radar site by its ICAO identifier (case-insensitive).
///
/// # Example
///
/// ```
/// let site = nexrad::site("KTLX").unwrap();
/// assert_eq!(site.city, "Oklahoma City");
/// ```
#[cfg(feature = "model")]
pub fn site(id: &str) -> Option<&'static model::meta::registry::SiteEntry> {
    model::meta::registry::site_by_id(id)
}

/// Find the nearest NEXRAD site to a given latitude/longitude.
///
/// # Example
///
/// ```
/// let site = nexrad::nearest_site(35.4676, -97.5164).unwrap();
/// assert_eq!(site.id, "KTLX");
/// ```
#[cfg(feature = "model")]
pub fn nearest_site(
    latitude: f32,
    longitude: f32,
) -> Option<&'static model::meta::registry::SiteEntry> {
    model::meta::registry::nearest_site(latitude, longitude)
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
