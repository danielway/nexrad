//!
//! # Real-time NEXRAD Data
//! Near real-time (within seconds) NEXRAD radar data is uploaded to an AWS S3 bucket by NOAA. This
//! data is organized into a series of "volumes", each containing ~55 "chunks". This module provides
//! functions for identifying the most recent volume with data for a specified radar site and
//! downloading the chunks within that volume.
//!
//! A fixed number (999) volumes exist in the S3 bucket which are rotated through in a round-robin
//! fashion. Chunks are added to each volume approximately every 3 seconds with very little latency
//! from the data's collection time. TODO: quantify this using the message timestamps.
//!
//! There may be gaps in the volume data, as illustrated in the real example below from KDMX:
//! ```text
//! Volume 001: 2024-08-04 10:10:07 UTC
//! ...
//! Volume 085: 2024-08-04 17:10:49 UTC
//! Volume 086: No files found.
//! ...
//! Volume 670: No files found.
//! Volume 671: 2024-08-03 00:00:21 UTC
//! ...
//! Volume 999: 2024-08-04 10:06:37 UTC
//! ```
//! The [get_latest_volume] function will find the volume with the most recent data using a binary
//! search approach to minimize the number of network calls made. Once the latest volume is found
//! for a session, a different routine should be used to poll new data for that volume and advance
//! to the next volume when the active one is filled.
//!

mod volume_index;
pub use volume_index::*;

mod chunk;
pub use chunk::*;

mod chunk_type;
pub use chunk_type::*;

mod chunk_identifier;
pub use chunk_identifier::*;

mod download_chunk;
pub use download_chunk::*;

mod get_latest_volume;
pub use get_latest_volume::*;

mod list_chunks_in_volume;
pub use list_chunks_in_volume::*;

mod estimate_next_chunk_time;
pub use estimate_next_chunk_time::*;

mod search;

const REALTIME_BUCKET: &str = "unidata-nexrad-level2-chunks";
