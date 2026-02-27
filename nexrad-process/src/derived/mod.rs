//! Derived products computed from radar scans.
//!
//! These algorithms produce new data products by combining information from
//! multiple elevation sweeps or by applying transformations that require
//! additional context beyond a single sweep.

mod composite;
mod srvel;
mod vertical;

pub use composite::CompositeReflectivity;
pub use srvel::StormRelativeVelocity;
pub use vertical::VerticalCrossSection;
