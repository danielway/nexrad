//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod decode;
pub mod file;
pub mod model;
pub mod result;

#[cfg(any(feature = "decompress", feature = "decompress-wasm"))]
pub mod decompress;

#[cfg(feature = "download")]
pub mod download;
