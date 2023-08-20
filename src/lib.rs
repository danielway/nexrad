//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod chunk;
pub mod decode;
pub mod decompress;
pub mod file;
pub mod result;

#[cfg(feature = "download")]
pub mod download;
