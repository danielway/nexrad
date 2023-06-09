//! # NEXRAD
//!
//! Download and decode functions for NEXRAD radar data.
//!

extern crate core;

pub mod chunk;
pub mod fetch;
pub mod cache;
pub mod decompress;
pub mod decode;
pub mod result;