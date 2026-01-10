//! Binary protocol decoding for NEXRAD Archive II data.
//!
//! This crate provides low-level decoding functions and type definitions for NEXRAD
//! weather radar data, implementing NOAA's WSR-88D Interface Control Document for
//! Archive II (ICD 2620010H, Build 19.0).
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
//! - `nexrad-model` - Convert decoded messages to [`nexrad_model`] types
//! - `uom` - Type-safe units of measure for physical quantities
//!
//! # Module Organization
//!
//! - [`messages`] - Message parsing and type definitions
//! - [`summarize`] - Utilities for summarizing message collections
//! - [`binary_data`] - Wrapper type for binary blobs with debug support

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]

pub mod binary_data;
pub mod messages;
pub mod result;
pub mod summarize;

mod slice_reader;
mod util;
