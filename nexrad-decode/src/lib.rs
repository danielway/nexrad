//! Binary protocol decoding for NEXRAD Archive II data.
//!
//! This crate provides low-level decoding functions and type definitions for NEXRAD
//! weather radar data, implementing NOAA's WSR-88D Interface Control Document for the
//! RDA/RPG (ICD 2620002AA, Build 24.0).
//!
//! # Overview
//!
//! The main entry point is [`messages::decode_messages`], which parses binary data
//! into structured [`messages::Message`] objects:
//!
//! ```ignore
//! use nexrad_decode::messages::{decode_messages, MessageContents};
//!
//! let messages = decode_messages(&decompressed_data)?;
//!
//! for message in &messages {
//!     match message.contents() {
//!         MessageContents::DigitalRadarData(data) => {
//!             // Process radar data message
//!         }
//!         MessageContents::VolumeCoveragePattern(vcp) => {
//!             // Process VCP information
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! With the `model` feature, `model::ScanBuilder` assembles decoded messages into a
//! `nexrad_model::data::Scan`, converting radar data messages to radials and taking the
//! volume coverage pattern and site from the first messages that carry them.
//!
//! # Message Types
//!
//! The decoder handles several message types defined in the ICD:
//!
//! - **Digital Radar Data (Type 31)** - Primary radar measurements
//! - **RDA Status Data (Type 2)** - Radar status and alarms
//! - **Volume Coverage Pattern (Type 5)** - Scanning strategy details
//! - **Clutter Filter Map (Type 15)** - Clutter suppression data
//!
//! # Features
//!
//! - `model` (default) - Convert decoded messages to `nexrad_model` types
//! - `uom` - Type-safe units of measure for physical quantities
//!
//! # Module Organization
//!
//! - [`messages`] - Message parsing and type definitions
//! - [`summarize`] - Utilities for summarizing message collections
//! - [`result`] - Error and result types ([`Error`] and [`Result`] are re-exported at the root)
//!
//! # Crate Boundaries
//!
//! This crate provides **binary protocol parsing** with the following responsibilities
//! and constraints:
//!
//! ## Responsibilities
//!
//! - ✓ Parse binary data per NOAA ICD 2620002AA specification
//! - ✓ Convert raw bytes into structured message types
//! - ✓ Provide conversion to `nexrad_model` types (when feature enabled)
//! - ✓ Validate message structures
//!
//! ## Constraints
//!
//! - ✗ **No I/O operations** (operates on byte slices provided by caller)
//! - ✗ **No file or network access**
//! - ✗ **No rendering or visualization**
//! - ✗ **No CLI or user interaction**
//!
//! This crate is purely concerned with parsing binary protocol data. It accepts byte
//! slices as input and returns structured data types. All I/O operations (reading files,
//! downloading from AWS, decompression) are handled by the `nexrad-data` crate.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

/// Wrapper type for binary data with debug formatting support.
pub(crate) mod binary_data;
/// Message parsing and type definitions for NEXRAD Level II data.
pub mod messages;
#[cfg(feature = "model")]
pub mod model;
/// Result and error types for decoding operations.
pub mod result;
pub use result::{Error, Result};
/// Utilities for summarizing collections of decoded messages.
pub mod summarize;

mod segmented_slice_reader;
mod slice_reader;
mod util;
