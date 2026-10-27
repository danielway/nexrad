//! Core data model for NEXRAD weather radar data.
//!
//! This crate provides an ergonomic API for working with NEXRAD radar data, documented
//! for users who may not be familiar with the NOAA Archive II format.
//!
//! # Overview
//!
//! The data model consists of:
//!
//! - [`data::Scan`] - A complete radar scan containing multiple sweeps
//! - [`data::Sweep`] - A single rotation at one elevation angle
//! - [`data::Radial`] - A single beam direction with moment data
//! - [`data::MomentData`] - Gate-by-gate measurements for a product
//! - [`meta::Site`] - Radar site metadata (location, identifier)
//!
//! # Example
//!
//! ```
//! use nexrad_model::data::{MomentValue, Product, Scan};
//!
//! fn process_scan(scan: &Scan) {
//!     println!("{}", scan.coverage_pattern_number());
//!
//!     for sweep in scan.sweeps() {
//!         for radial in sweep.radials() {
//!             if let Some(reflectivity) = radial.moment(Product::Reflectivity) {
//!                 for value in reflectivity.iter() {
//!                     match value {
//!                         MomentValue::Value(dbz) => println!("dBZ: {}", dbz),
//!                         MomentValue::BelowThreshold => {}
//!                         MomentValue::RangeFolded => {}
//!                         // Only produced for Product::ClutterFilterPower
//!                         MomentValue::CfpStatus(_) => {}
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Features
//!
//! Optional features provide additional functionality:
//!
//! - `serde` - Serialization/deserialization support via [`serde`](https://docs.rs/serde).
//!   The data, geometry and site types implement `Serialize` and `Deserialize`; the builders,
//!   `SiteEntry` and `Error` do not.
//!
//! - `chrono` - Date/time support via [`chrono`](https://docs.rs/chrono).
//!   Enables `collection_time()` returning `DateTime<Utc>` instead of raw timestamps.
//!
//! Accessors return plain numbers with the unit in the name (`elevation_angle_degrees()`,
//! `gate_interval_km()`). Unit-typed accessors (for example with the `uom` crate) may be offered
//! later as a separate crate or extension trait.
//!
//! # Crate Boundaries
//!
//! This crate is a **pure data model** with the following responsibilities and constraints:
//!
//! ## Responsibilities
//!
//! - ✓ Define domain types (Scan, Sweep, Radial, Site, MomentData)
//! - ✓ Provide data transformations and validations
//! - ✓ Support optional features (serde, chrono)
//!
//! ## Constraints
//!
//! - ✗ **No I/O operations** (file, network, stdio)
//! - ✗ **No binary parsing or encoding**
//! - ✗ **No rendering or visualization**
//! - ✗ **No CLI or user interaction**
//!
//! This crate focuses solely on providing ergonomic data structures for working with
//! NEXRAD radar data. All I/O, parsing, and rendering concerns are handled by separate
//! crates in the NEXRAD library suite.

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
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

mod binary_data;

pub mod data;
pub mod geo;
pub mod meta;
pub mod result;
pub use result::{Error, Result};
