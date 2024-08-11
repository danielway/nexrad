//!
//! This module contains models representing digital radar data collected by the NEXRAD weather
//! radar network. These models and their APIs are intended to be ergonomic, understandable, and
//! performant. They do not exactly match the encoded structure from common archival formats.
//!
//! Optionally, the `uom` feature provides APIs that use the `uom` crate for type-safe units of
//! measure.
//!

mod sweep;
pub use sweep::*;

mod scan;
pub use scan::*;

mod radial;

pub use radial::*;

mod moment;
pub use moment::*;
