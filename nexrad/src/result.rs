//!
//! Unified error types for the NEXRAD facade crate.
//!
//! This module provides a unified error type [`Error`] that consolidates errors from all
//! sub-crates (nexrad-model, nexrad-decode, nexrad-data, nexrad-render) into a single
//! error surface for users of the facade crate.
//!
//! ## Error Hierarchy
//!
//! The [`Error`] enum has variants corresponding to each enabled sub-crate feature.
//! See the [`Error`] type documentation for details on specific variants.
//!
//! ## Automatic Conversion
//!
//! All sub-crate error types automatically convert to the unified [`Error`] type via
//! [`From`] trait implementations, enabling seamless error propagation with the `?` operator:
//!
//! ```no_run
//! # use nexrad::Result;
//! fn example() -> Result<()> {
//!     // All sub-crate errors automatically convert to nexrad::Error
//!     let data = std::fs::read("volume.ar2")
//!         .map_err(nexrad::data::result::Error::from)?;  // io::Error -> data::Error -> nexrad::Error
//!     let volume = nexrad::data::volume::File::new(data);
//!     let scan = volume.scan()?;  // nexrad_data::result::Error automatically converts
//!     Ok(())
//! }
//! ```
//!
//! ## Feature-Gated Variants
//!
//! Error variants are conditionally compiled based on the enabled features:
//!
//! - `model` feature enables the Model variant
//! - `decode` feature enables the Decode variant
//! - `data` feature enables the Data variant
//! - `render` feature enables the Render variant
//!
//! All features are enabled by default.
//!
//! ## Error Source Chain
//!
//! The unified error type preserves the complete error source chain. You can inspect
//! the underlying error using the [`std::error::Error::source`] method:
//!
//! ```no_run
//! # use std::error::Error as StdError;
//! # use nexrad::Result;
//! # fn example() -> Result<()> {
//! #     let data = std::fs::read("volume.ar2")
//! #         .map_err(nexrad::data::result::Error::from)?;
//! #     let volume = nexrad::data::volume::File::new(data);
//! #     let scan = volume.scan()?;
//! #     Ok(())
//! # }
//! match example() {
//!     Err(err) => {
//!         eprintln!("Error: {}", err);
//!         if let Some(source) = err.source() {
//!             eprintln!("Caused by: {}", source);
//!         }
//!     }
//!     Ok(_) => {}
//! }
//! ```

use thiserror::Error as ThisError;

/// A unified result type using the facade-level [`Error`].
///
/// This is a convenience type alias that uses the unified error type for all operations
/// exposed through the nexrad facade crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for the NEXRAD facade crate.
///
/// This error type consolidates errors from all sub-crates into a single error surface,
/// making it easier to handle errors when using multiple sub-crates together.
///
/// Each variant wraps the error type from the corresponding sub-crate, preserving the
/// full error information and source chain.
#[derive(ThisError, Debug)]
pub enum Error {
    /// Error from the data model layer (nexrad-model).
    ///
    /// This variant is available when the `model` feature is enabled (default).
    ///
    /// Model errors typically indicate issues with data transformations or validations,
    /// such as mismatched elevation angles when combining sweeps.
    #[cfg(feature = "model")]
    #[error("model error: {0}")]
    Model(#[from] nexrad_model::result::Error),

    /// Error from binary protocol decoding (nexrad-decode).
    ///
    /// This variant is available when the `decode` feature is enabled (default).
    ///
    /// Decode errors indicate issues parsing the binary NEXRAD Level II message format,
    /// such as I/O errors reading data, invalid message structures, or missing required
    /// fields like collection timestamps.
    #[cfg(feature = "decode")]
    #[error("decode error: {0}")]
    Decode(#[from] nexrad_decode::result::Error),

    /// Error from data access and I/O operations (nexrad-data).
    ///
    /// This variant is available when the `data` feature is enabled (default).
    ///
    /// Data errors encompass file I/O, decompression, AWS S3 operations, and any errors
    /// from the underlying decode and model layers that occur during data access.
    #[cfg(feature = "data")]
    #[error("data error: {0}")]
    Data(#[from] nexrad_data::result::Error),

    /// Error from rendering and visualization (nexrad-render).
    ///
    /// This variant is available when the `render` feature is enabled (default).
    ///
    /// Render errors indicate issues with image rendering, such as missing radar products
    /// in the data or failures in the underlying graphics backend.
    #[cfg(feature = "render")]
    #[error("render error: {0}")]
    Render(#[from] nexrad_render::result::Error),

    /// I/O error from file operations.
    ///
    /// This variant wraps standard library I/O errors that occur when reading
    /// volume files from disk.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// No data available for the requested site and date/time.
    ///
    /// This error occurs when attempting to download archive data that doesn't exist,
    /// such as requesting a date before radar operations began or a future date.
    #[error("no data available for site {site} on {date}")]
    NoDataAvailable {
        /// The radar site identifier (e.g., "KTLX").
        site: String,
        /// The requested date in YYYY-MM-DD format.
        date: String,
    },
}
