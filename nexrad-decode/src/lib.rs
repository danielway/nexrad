//!
//! # nexrad-decode
//! Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are
//! in accordance with NOAA's WSR-88D Interface Control Document for Archive II "ICD 2620010H"
//! build 19.0.
//!
//! Optionally, the `nexrad-model` feature provides mappings to a common model for representing
//! radar data. The `uom` feature can also be used to provide type-safe units of measure.
//!

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]

pub mod messages;
pub mod result;
pub mod summarize;

mod reader;
mod util;
