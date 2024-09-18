//!
//! # nexrad-data
//! Provides structure definitions and decoding functions for NEXRAD Archive II volume files, along
//! with functions for downloading both archival and real-time data from open cloud providers like
//! AWS OpenData.
//!

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

#[cfg(feature = "aws")]
pub mod aws;

pub mod volume;

pub mod result;
