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
//! - `nexrad-render` - Visualization and rendering
//!
//! ## Features
//!
//! All features are enabled by default. You can disable default features and
//! enable only what you need:
//!
//! - `model` - Core data model types
//! - `decode` - Protocol decoding
//! - `data` - Data access and AWS integration
//! - `render` - Visualization and rendering
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
//! ## Error Handling
//!
//! This crate provides unified error types via [`Error`] and [`Result<T>`]. All sub-crate
//! errors automatically convert to the unified type via `From` traits, enabling seamless
//! error propagation:
//!
//! ```ignore
//! fn process_volume() -> nexrad::Result<()> {
//!     let data = std::fs::read("volume.ar2")?;  // io::Error converts
//!     let volume = nexrad::data::volume::File::new(data);
//!     let scan = volume.scan()?;  // data/decode/model errors convert
//!     Ok(())
//! }
//! ```
//!
//! See the [`result`] module for detailed error handling documentation.
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
//!
//! ## Crate Responsibility Boundaries
//!
//! This facade crate enforces clear separation of concerns across the library suite:
//!
//! ### Re-exported Crates (Part of Public API)
//!
//! - **`nexrad-model`**: Pure data structures and transformations
//!   - ✓ Domain types (Scan, Sweep, Radial, Site)
//!   - ✓ Data transformations and validations
//!   - ✗ No I/O operations (file, network, stdio)
//!   - ✗ No binary parsing or encoding
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-decode`**: Binary protocol parsing
//!   - ✓ Parsing NEXRAD Level II message format (NOAA ICD 2620010H)
//!   - ✓ Conversion to model types (when feature enabled)
//!   - ✗ No I/O operations (operates on byte slices)
//!   - ✗ No file or network access
//!   - ✗ No rendering or visualization
//!
//! - **`nexrad-data`**: File I/O and network access
//!   - ✓ Archive II file handling (including limited volume header decoding)
//!   - ✓ AWS S3 integration (when `aws` feature enabled)
//!   - ✓ Decompression and format handling
//!   - ✓ Uses `nexrad-decode` for message parsing
//!   - ✓ Uses `nexrad-model` for high-level types
//!   - ✗ No rendering or visualization
//!   - ✗ No CLI or user interaction
//!
//! - **`nexrad-render`**: Visualization and image rendering
//!   - ✓ Render radar data to in-memory images
//!   - ✓ Apply color scales to moment data
//!   - ✓ Consume `nexrad-model` types
//!   - ✗ No I/O operations
//!   - ✗ No data access or parsing

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

pub mod result;

pub use result::{Error, Result};

/// Re-export of `nexrad-model` for core data types.
#[cfg(feature = "model")]
pub use nexrad_model as model;

/// Re-export of `nexrad-decode` for binary protocol decoding.
#[cfg(feature = "decode")]
pub use nexrad_decode as decode;

/// Re-export of `nexrad-data` for data access and AWS integration.
#[cfg(feature = "data")]
pub use nexrad_data as data;

/// Re-export of `nexrad-render` for visualization and rendering.
#[cfg(feature = "render")]
pub use nexrad_render as render;
