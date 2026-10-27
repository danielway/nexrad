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
//! [`MessageContents`] lists the decoded message types; any other type decodes as
//! [`MessageContents::Other`] with its raw payload.
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
//! 4. Unit suffixes are `_degrees`, `_degrees_per_second`, `_km`, `_meters`, `_mps`, `_kw`,
//!    `_watts`, `_bytes`, `_halfwords`, `_db`, `_dbm` and `_db_per_km`.
//!
//! The RDA adaptation data (Type 18) and performance/maintenance data (Type 3) accessors are
//! named after the ICD table mnemonics, without unit suffixes, and these names are fixed for
//! 1.x. The ICD tables for those messages are the reference for each field's meaning and unit.
//!
//! # Features
//!
//! - `model` (default) - Convert decoded messages to `nexrad_model` types
//!
//! Accessors return plain numbers with the unit in the name. Unit-typed accessors (for example
//! with the `uom` crate) may be offered later as a separate crate or extension trait.
//!
//! # Module Organization
//!
//! - [`messages`] - Message parsing and type definitions (the entry points and core message
//!   types are re-exported at the root)
//! - [`result`] - Error and result types ([`Error`] and [`Result`] are re-exported at the root)
//!
//! # Crate Boundaries
//!
//! This crate performs no I/O and no decompression: it decodes byte slices the caller has
//! already decompressed. Reading files, decompressing LDM records and downloading from AWS are
//! done by `nexrad-data`.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unnameable_types)]

// Compiles the README's code blocks as doc tests.
#[cfg(all(doctest, feature = "model"))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

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
