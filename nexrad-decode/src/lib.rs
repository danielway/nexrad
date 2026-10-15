//! Binary protocol decoding for NEXRAD Archive II data.
//!
//! This crate provides low-level decoding functions and type definitions for NEXRAD
//! weather radar data, implementing NOAA's WSR-88D Interface Control Document for the
//! RDA/RPG (ICD 2620002AA, Build 24.0).
//!
//! # Overview
//!
//! The main entry point is [`decode_messages`], which parses a sequence of messages, such as a
//! decompressed Archive II record, into structured [`Message`] values. Decoding is strict: it
//! returns the error for the first message that cannot be decoded, along with its offset.
//!
//! ```no_run
//! use nexrad_decode::{decode_messages, MessageContents};
//!
//! # fn run(decompressed_data: Vec<u8>) -> nexrad_decode::Result<()> {
//! for message in decode_messages(&decompressed_data)? {
//!     match message.contents() {
//!         MessageContents::DigitalRadarData(data) => {
//!             // Process radar data message
//!         }
//!         MessageContents::RdaVolumeCoveragePattern(vcp) => {
//!             // Process VCP information
//!         }
//!         MessageContents::Other { message_type, data } => {
//!             // A message type this crate does not decode, with its raw payload
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! [`iter_messages`] yields one `Result` per message instead, so a caller can keep the messages
//! decoded before a failure.
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
//! # Accessor Naming
//!
//! Accessors on the decoded message types follow one convention:
//!
//! 1. An accessor's stem is the ICD field name (`azimuth_angle`, `unambiguous_range`,
//!    `rda_status`, ...). Raw and decoded accessors for the same field share the stem.
//! 2. When the wire encoding differs from the decoded value (a scaled integer, a code, a
//!    bitfield, a byte array), `<stem>_raw()` returns the wire value. The decoded value is
//!    `<stem>()` when it is an enum, struct or date/time, and `<stem>_<unit>()` when it is a
//!    plain number with a unit, e.g. `unambiguous_range_raw()` and `unambiguous_range_km()`,
//!    or `vcp_number_raw()` and `vcp_number()`. A byte string is `<stem>_raw()` and
//!    `<stem>() -> Option<&str>`, `None` when the bytes are not valid UTF-8
//!    (`radar_identifier_raw()` and `radar_identifier()`). A field split across a date and a
//!    time decodes to one `<stem>_date_time()` (`map_generation_date_time()`).
//! 3. When the wire value already is the decoded value, there is no `_raw()` accessor. The
//!    accessor is `<stem>_<unit>()` if the field has a unit (`azimuth_angle_degrees()`), and
//!    the bare stem for dimensionless numbers, versions and flags (`azimuth_number()`).
//!    Counts are `<thing>_count()` (`data_block_count()`, `elevation_cut_count()`).
//! 4. With the `uom` feature, `<stem>()` with no suffix returns an `f64` `uom` quantity
//!    (`azimuth_angle()`, `unambiguous_range()`). Where the bare stem already names an enum
//!    (`azimuth_resolution_spacing()`), there is no `uom` accessor.
//! 5. Unit suffixes are `_degrees`, `_degrees_per_second`, `_km`, `_meters`, `_mps`, `_kw`,
//!    `_watts`, `_bytes`, `_halfwords`, `_db`, `_dbm` and `_db_per_km`.
//!
//! The RDA adaptation data (Type 18) and performance/maintenance data (Type 3) accessors
//! currently keep the ICD table mnemonics without unit suffixes.
//!
//! # Features
//!
//! - `model` (default) - Convert decoded messages to `nexrad_model` types
//! - `uom` - Type-safe units of measure for physical quantities
//!
//! # Module Organization
//!
//! - [`messages`] - Message parsing and type definitions (the entry points and core message
//!   types are re-exported at the root)
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

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![warn(unnameable_types)]

/// Wrapper type for binary data with debug formatting support.
pub(crate) mod binary_data;
/// Message parsing and type definitions for NEXRAD Level II data.
pub mod messages;
pub use messages::{
    decode_messages, iter_messages, Message, MessageContents, MessageIter, MessageType,
};
#[cfg(feature = "model")]
pub mod model;
/// Result and error types for decoding operations.
pub mod result;
pub use result::{Error, Result};

mod segmented_slice_reader;
mod slice_reader;
mod util;
