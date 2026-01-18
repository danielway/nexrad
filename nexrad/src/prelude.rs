//! Convenient re-exports with unified terminology.
//!
//! This module provides ergonomic imports for common usage patterns, including
//! type aliases that align with standard radar terminology.
//!
//! # Example
//!
//! ```ignore
//! use nexrad::prelude::*;
//!
//! let volume = nexrad::load_file("radar.ar2v")?;
//! for sweep in volume.sweeps() {
//!     for radial in sweep.radials() {
//!         if let Some(reflectivity) = radial.reflectivity() {
//!             println!("{} gates", reflectivity.gate_count());
//!         }
//!     }
//! }
//! ```
//!
//! # Terminology
//!
//! | Term | Underlying Type | Description |
//! |------|-----------------|-------------|
//! | `Volume` | `Scan` | Complete radar scan at multiple elevations |
//! | `Sweep` | `Sweep` | Single rotation at one elevation angle |
//! | `Radial` | `Radial` | Single beam direction with moment data |
//! | `Moment` | `MomentData` | Per-gate measurements for one data type |
//! | `GateValue` | `MomentValue` | Individual range bin measurement |
//! | `VCP` | `VolumeCoveragePattern` | Scanning strategy configuration |

pub use crate::{Error, Result};

// Type aliases for unified terminology
#[cfg(feature = "model")]
pub use nexrad_model::data::Scan as Volume;

#[cfg(feature = "model")]
pub use nexrad_model::data::MomentData as Moment;

#[cfg(feature = "model")]
pub use nexrad_model::data::MomentValue as GateValue;

#[cfg(feature = "model")]
pub use nexrad_model::data::VolumeCoveragePattern as VCP;

// Re-export types that don't need aliases
#[cfg(feature = "model")]
pub use nexrad_model::data::{Radial, RadialStatus, Sweep};

#[cfg(feature = "model")]
pub use nexrad_model::meta::Site;

// Top-level functions for loading volumes
#[cfg(all(feature = "data", feature = "model"))]
pub use crate::{load, load_file};

// Async download functions (require aws feature)
#[cfg(all(feature = "data", feature = "model", feature = "aws"))]
pub use crate::{download_at, download_latest};

#[cfg(all(feature = "data", feature = "aws"))]
pub use crate::list_volumes;
