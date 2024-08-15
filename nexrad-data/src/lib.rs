//!
//! # nexrad-data
//! Download and processing functions for NEXRAD weather radar data.
//!

#[cfg(feature = "aws")]
pub mod aws;

pub mod archive;
pub mod realtime;

pub mod result;
