//!
//! Unified error types for the NEXRAD facade crate.
//!
//! This module provides a unified error type [`Error`] that consolidates errors from all
//! sub-crates (nexrad-model, nexrad-decode, nexrad-data, nexrad-render, nexrad-process) into a
//! single error surface for users of the facade crate.
//!
//! ## Error Hierarchy
//!
//! Each [`Error`] variant names the layer an error originated in, not the layer it passed
//! through on its way up:
//!
//! - `Error::Model` wraps `nexrad_model` errors.
//! - `Error::Decode` wraps `nexrad_decode` errors.
//! - `Error::Data` wraps `nexrad_data` errors that are specific to data access, such as
//!   decompression, archive layout and AWS failures.
//! - `Error::Render` wraps `nexrad_render` errors.
//! - `Error::Process` wraps `nexrad_process` errors.
//! - [`Error::Io`] wraps [`std::io::Error`], whether it comes from the facade itself (for
//!   example `load_file`) or from `nexrad_data`.
//! - `Error::NoDataAvailable` reports that an archive lookup found no matching file.
//!
//! ## Automatic Conversion
//!
//! Every sub-crate error type converts to the unified [`Error`] via [`From`], so the `?`
//! operator propagates them directly. Converting a `nexrad_data` error flattens the
//! variants that only wrap another layer's error: `data::Error::Decode(e)` becomes
//! `Error::Decode(e)`, `data::Error::Model(e)` becomes `Error::Model(e)` and
//! `data::Error::Io(e)` becomes `Error::Io(e)`. All other `nexrad_data` errors become
//! `Error::Data`.
//!
//! ```no_run
//! # #[cfg(feature = "data")]
//! fn example() -> nexrad::Result<()> {
//!     let bytes = std::fs::read("volume.ar2")?; // io::Error -> Error::Io
//!     let volume = nexrad::data::volume::File::new(bytes).decompress()?;
//!     let scan = volume.scan()?; // data::Error -> Error::Data, Error::Decode, ...
//!     println!("{} sweeps", scan.sweeps().len());
//!     Ok(())
//! }
//! ```
//!
//! ## Feature-Gated Variants
//!
//! Error variants are conditionally compiled based on the enabled features:
//!
//! - `model` enables `Error::Model`
//! - `decode` enables `Error::Decode`
//! - `data` enables `Error::Data`
//! - `render` enables `Error::Render`
//! - `process` enables `Error::Process`
//! - `aws` enables `Error::NoDataAvailable`
//!
//! [`Error::Io`] is always available. Features that build on others enable them too (for
//! example, `data` also enables `decode` and `model`), so the flattened variants always exist
//! when `data` is enabled. All features are enabled by default.
//!
//! ## Error Source Chain
//!
//! Each wrapping variant reports the wrapped error as its [`std::error::Error::source`], and its
//! own `Display` text names only the layer (`decode error`), so walk the source chain for the
//! details:
//!
//! ```no_run
//! # #[cfg(feature = "data")]
//! # fn main() {
//! use std::error::Error as _;
//!
//! if let Err(err) = nexrad::load_file("volume.ar2") {
//!     eprintln!("Error: {err}");
//!     let mut source = err.source();
//!     while let Some(cause) = source {
//!         eprintln!("Caused by: {cause}");
//!         source = cause.source();
//!     }
//! }
//! # }
//! # #[cfg(not(feature = "data"))]
//! # fn main() {}
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
/// Each wrapping variant names the layer the error originated in and preserves the wrapped
/// error as its source. Decode, model and I/O errors that reach the facade through
/// `nexrad_data` are flattened into `Error::Decode`, `Error::Model` and [`Error::Io`]; see the
/// [module documentation](self) for details.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error from the data model layer (nexrad-model).
    ///
    /// This variant is available when the `model` feature is enabled (default).
    ///
    /// Model errors typically indicate issues with data transformations or validations,
    /// such as mismatched elevation angles when combining sweeps. Model errors raised
    /// inside `nexrad_data` also surface here.
    #[cfg(feature = "model")]
    #[error("model error")]
    Model(#[from] nexrad_model::result::Error),

    /// Error from binary protocol decoding (nexrad-decode).
    ///
    /// This variant is available when the `decode` feature is enabled (default).
    ///
    /// Decode errors indicate issues parsing the binary NEXRAD Level II message format,
    /// such as truncated data, invalid message structures, or missing required fields like
    /// collection timestamps. Decode errors raised inside `nexrad_data` also surface here.
    #[cfg(feature = "decode")]
    #[error("decode error")]
    Decode(#[from] nexrad_decode::result::Error),

    /// Error specific to data access (nexrad-data).
    ///
    /// This variant is available when the `data` feature is enabled (default).
    ///
    /// Data errors cover decompression, Archive II volume and record layout, real-time chunk
    /// handling and AWS S3 operations. Decode, model and I/O errors from `nexrad_data` are
    /// not wrapped here; they surface as `Error::Decode`, `Error::Model` and `Error::Io`.
    #[cfg(feature = "data")]
    #[error("data error")]
    Data(#[source] nexrad_data::result::Error),

    /// Error from rendering and visualization (nexrad-render).
    ///
    /// This variant is available when the `render` feature is enabled (default).
    ///
    /// Render errors indicate issues with image rendering or failures in the underlying
    /// graphics backend.
    #[cfg(feature = "render")]
    #[error("render error")]
    Render(#[from] nexrad_render::result::Error),

    /// Error from radar data processing algorithms (nexrad-process).
    ///
    /// This variant is available when the `process` feature is enabled (default).
    ///
    /// Process errors indicate issues with processing algorithms, such as invalid
    /// field geometry, missing required data, or invalid parameter values.
    #[cfg(feature = "process")]
    #[error("process error")]
    Process(#[from] nexrad_process::result::Error),

    /// I/O error.
    ///
    /// This variant wraps standard library I/O errors, both those raised by the facade
    /// itself (for example when `load_file` reads from disk) and those raised inside
    /// `nexrad_data`.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// No data available for the requested site and date/time.
    ///
    /// This variant is available when the `aws` feature is enabled (default).
    ///
    /// This error occurs when attempting to download archive data that doesn't exist,
    /// such as requesting a date before radar operations began or a future date.
    #[cfg(feature = "aws")]
    #[error("no data available for site {site} on {date}")]
    NoDataAvailable {
        /// The radar site identifier (e.g., "KTLX").
        site: String,
        /// The requested date.
        date: chrono::NaiveDate,
    },
}

#[cfg(feature = "data")]
impl From<nexrad_data::result::Error> for Error {
    fn from(error: nexrad_data::result::Error) -> Self {
        match error {
            nexrad_data::result::Error::Decode(error) => Error::Decode(error),
            nexrad_data::result::Error::Model(error) => Error::Model(error),
            nexrad_data::result::Error::Io(error) => Error::Io(error),
            other => Error::Data(other),
        }
    }
}
