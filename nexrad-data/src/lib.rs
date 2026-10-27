//! Data access for NEXRAD weather radar files.
//!
//! This crate provides tools for working with NEXRAD Archive II volume files and
//! accessing data from cloud providers like AWS.
//!
//! # Overview
//!
//! - [`volume`] - Parse and decode local Archive II volume files
//! - `aws` - Download data from AWS (archive and real-time, requires `aws` feature)
//!
//! # Working with Local Files
//!
//! ```no_run
//! use nexrad_data::volume::File;
//!
//! let data = std::fs::read("KTLX20130520_201643_V06.ar2v")?;
//! let volume = File::new(data).decompress()?;
//!
//! // Access the volume header
//! if let Some(header) = volume.header() {
//!     println!("Volume date: {:?}", header.date_time());
//! }
//!
//! // Process LDM records
//! for mut record in volume.records()? {
//!     if record.compressed() {
//!         record = record.decompress()?;
//!     }
//!     let messages = record.messages()?;
//!     // Process radar messages...
//! }
//! # Ok::<(), nexrad_data::Error>(())
//! ```
//!
//! # Downloading from AWS
//!
//! The `aws` feature (enabled by default) provides access to NOAA's NEXRAD data on AWS:
//!
//! - **Archive data**: Historical volumes from the NEXRAD Level II archive
//! - **Real-time data**: Live radar data with minimal latency, via `aws::realtime::chunk_stream`
//!   (a stream) or `aws::realtime::ChunkIterator` (pull-based)
//!
//! ```no_run
//! # #[cfg(feature = "aws")]
//! # async fn run() -> nexrad_data::Result<()> {
//! use nexrad_data::aws::archive;
//! use chrono::NaiveDate;
//!
//! // List available files for a date and site
//! let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
//! let files = archive::list_files("KTLX", date).await?;
//!
//! // Download a specific file
//! let file = archive::download_file(&files[0]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `aws` (default) - Enable AWS S3 data access, including `aws::realtime::chunk_stream` for polling
//!   real-time data (requires an async runtime; WASM-compatible)
//! - `model` (default) - Enable conversion to the high-level `Scan` model (`volume::File::scan`,
//!   `aws::realtime::assemble_volume`)
//! - `parallel` - Parallelize LDM record decompression and decoding using Rayon
//!
//! # Crate Boundaries
//!
//! This crate provides **data access and I/O operations** with the following
//! responsibilities and constraints:
//!
//! ## Responsibilities
//!
//! - ✓ Read and parse Archive II volume files
//! - ✓ Handle file I/O operations
//! - ✓ Decompress bzip2-compressed LDM records
//! - ✓ AWS S3 integration for archive and real-time data (when `aws` feature enabled)
//! - ✓ Use `nexrad-decode` for message parsing and, with the `model` feature, for conversion to
//!   `nexrad-model` types (`volume::File::scan` and `aws::realtime::assemble_volume` hand decoded
//!   messages to `nexrad_decode::model::ScanBuilder`)
//! - ✓ Orchestrate the data pipeline from raw bytes to structured models
//!
//! ## Constraints
//!
//! - ✗ **No protocol-to-model mapping** (that lives in `nexrad-decode`)
//! - ✗ **No rendering or visualization**
//! - ✗ **No CLI or user interaction**
//!
//! This crate serves as the **I/O boundary layer** for the NEXRAD library suite. It
//! handles all file and network operations, decompression, and coordinates between
//! the low-level binary parsing (`nexrad-decode`) and high-level data models
//! (`nexrad-model`).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(clippy::correctness)]
#![warn(unnameable_types)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]

// Compiles the README's code blocks as doc tests.
#[cfg(all(doctest, feature = "model"))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

#[cfg(feature = "aws")]
/// AWS S3 integration for downloading NEXRAD data from the cloud.
pub mod aws;

/// Local Archive II volume file handling.
pub mod volume;

/// Result and error types for data operations.
pub mod result;
pub use result::{Error, Result};

#[cfg(feature = "aws")]
pub use result::AwsError;
