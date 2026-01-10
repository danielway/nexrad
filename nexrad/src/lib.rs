//! # nexrad
//!
//! A Rust library for working with NEXRAD (Next Generation Weather Radar) data.
//!
//! This crate serves as the main entry point for the NEXRAD library suite, providing
//! convenient re-exports from the underlying crates:
//!
//! - `nexrad-model` - Core data model types (Scan, Sweep, Radial, Site)
//! - `nexrad-decode` - Binary protocol decoding for Archive II format
//! - `nexrad-data` - Data access (local files, AWS S3)
//!
//! ## Features
//!
//! All features are enabled by default. You can disable default features and
//! enable only what you need:
//!
//! - `model` - Core data model types
//! - `decode` - Protocol decoding
//! - `data` - Data access and AWS integration
//!
//! ## Quick Start
//!
//! ```ignore
//! use nexrad::data::volume::File;
//! use nexrad::model::data::{Scan, Sweep};
//!
//! // Load a local NEXRAD Archive II file
//! let data = std::fs::read("KTLX20130520_201643_V06.ar2v")?;
//! let volume = File::new(data);
//!
//! // Convert to the high-level data model
//! let scan: Scan = volume.scan()?;
//!
//! // Access sweeps and radials
//! for sweep in scan.sweeps() {
//!     println!("Elevation {}: {} radials",
//!         sweep.elevation_number(),
//!         sweep.radials().len());
//! }
//! ```
//!
//! ## Crate Organization
//!
//! For more specialized use cases, you can depend on individual crates directly:
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `nexrad-model` | Domain types with optional serde/chrono/uom support |
//! | `nexrad-decode` | Low-level binary parsing per NOAA ICD 2620010H |
//! | `nexrad-data` | Archive II file handling and AWS S3 access |
//! | `nexrad-render` | Visualization and image rendering |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

/// Re-export of `nexrad-model` for core data types.
#[cfg(feature = "model")]
pub use nexrad_model as model;

/// Re-export of `nexrad-decode` for binary protocol decoding.
#[cfg(feature = "decode")]
pub use nexrad_decode as decode;

/// Re-export of `nexrad-data` for data access and AWS integration.
#[cfg(feature = "data")]
pub use nexrad_data as data;
