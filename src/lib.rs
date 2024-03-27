//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod decode;
pub mod file;
pub mod model;
pub mod result;

#[cfg(feature = "decompress")]
pub mod decompress;

#[cfg(feature = "download")]
pub mod download;
