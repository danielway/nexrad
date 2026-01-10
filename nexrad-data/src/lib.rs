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
//! ```ignore
//! use nexrad_data::volume::File;
//!
//! let data = std::fs::read("KTLX20130520_201643_V06.ar2v")?;
//! let volume = File::new(data);
//!
//! // Access the volume header
//! if let Some(header) = volume.header() {
//!     println!("Volume date: {:?}", header.date());
//! }
//!
//! // Process LDM records
//! for record in volume.records() {
//!     let decompressed = record.decompress()?;
//!     let messages = decompressed.messages()?;
//!     // Process radar messages...
//! }
//! ```
//!
//! # Downloading from AWS
//!
//! The `aws` feature (enabled by default) provides access to NOAA's NEXRAD data on AWS:
//!
//! - **Archive data**: Historical volumes from the NEXRAD Level II archive
//! - **Real-time data**: Live radar data with minimal latency
//!
//! ```ignore
//! use nexrad_data::aws::archive;
//! use chrono::NaiveDate;
//!
//! // List available files for a date and site
//! let date = NaiveDate::from_ymd_opt(2023, 5, 20).unwrap();
//! let files = archive::list_files("KTLX", date).await?;
//!
//! // Download a specific file
//! let data = archive::download_file(&files[0]).await?;
//! ```
//!
//! # Features
//!
//! - `aws` - Enable AWS S3 data access (requires async runtime)
//! - `nexrad-model` - Enable conversion to high-level `Scan` model

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

#[cfg(feature = "aws")]
pub mod aws;

pub mod volume;

pub mod result;
