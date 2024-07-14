//!
//! This module contains models containing metadata about the radar data collected by the NEXRAD
//! weather network. This data may not change between radials, sweeps, or even scans, and thus it
//! is represented separately to avoid duplication in storage.
//!

/// A radar site's metadata including a variety of infrequently-changing properties.
pub struct Site {
    identifier: [u8; 4],
    latitude: f32,
    longitude: f32,
    height_meters: i16,
    feedhorn_height_meters: u16,
}
