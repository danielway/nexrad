//! Derived products computed from radar volume scans.
//!
//! These algorithms produce new data products by combining information from
//! multiple elevation sweeps or by applying transformations that require
//! additional context beyond a single sweep.

mod composite;
mod srvel;

pub use composite::CompositeReflectivity;
pub use srvel::StormRelativeVelocity;
