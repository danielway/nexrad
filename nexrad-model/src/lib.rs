//!
//! # nexrad-model
//! A common model for representing NEXRAD weather radar data. Provides an ergonomic API which is
//! documented for an audience who is not necessarily familiar with the NOAA Archive II format.
//!
//! A number of optional features are available:
//! - `uom`: Use the `uom` crate for type-safe units of measure.
//! - `serde`: Implement `serde::Serialize` and `serde::Deserialize` for all models.
//! - `chrono`: Use the `chrono` crate for date and time types.
//!

#![forbid(unsafe_code)]
#![warn(clippy::correctness)]
#![allow(clippy::too_many_arguments)]

pub mod data;
pub mod meta;
