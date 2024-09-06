//!
//! # nexrad-data
//! Download and processing functions for NEXRAD weather radar data.
//!

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

#[cfg(feature = "aws")]
pub mod aws;

pub mod volume;

pub mod result;
