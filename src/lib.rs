//!
//! # nexrad-data
//! Download and processing functions for NEXRAD weather radar data.
//!

#[cfg(feature = "aws")]
pub mod aws;

pub mod result;

mod file;
