use crate::aws::realtime::ChunkIdentifier;
use chrono::{DateTime, Utc};
use std::ops::Add;
use std::time::Duration;

/// Attempts to estimate the time at which the next chunk will be available given the previous chunk.
/// A None result indicates that the chunk should already be available.
#[cfg(feature = "nexrad-decode")]
pub fn estimate_next_chunk_time(
    previous_chunk: &ChunkIdentifier,
    volume_coverage_pattern: &nexrad_decode::messages::volume_coverage_pattern::Message,
) -> DateTime<Utc> {
    let previous_sequence = previous_chunk.sequence().unwrap_or(1);

    if previous_sequence >= 55 {
        return previous_chunk.date_time().unwrap_or_else(Utc::now);
    }

    let mut current_elevation_index = 0;
    let mut chunks_processed = 0;
    let mut index_within_elevation = 0;

    // Iterate through elevations to find where we are
    for (i, elevation) in volume_coverage_pattern.elevations.iter().enumerate() {
        // Determine number of chunks for this elevation based on super resolution
        let chunks_per_elevation = if elevation.super_resolution_control_half_degree_azimuth() {
            6 // (720 radials per sweep / 120 radials per chunk)
        } else {
            3 // (360 radials per sweep / 120 radials per chunk)
        };

        // Check if current sequence falls within this elevation
        if chunks_processed + chunks_per_elevation >= previous_sequence {
            current_elevation_index = i;
            index_within_elevation = previous_sequence - chunks_processed - 1; // 0-indexed
            break;
        }

        chunks_processed += chunks_per_elevation;
    }

    let next_chunk_waveform = if index_within_elevation + 1
        >= (if volume_coverage_pattern.elevations[current_elevation_index]
            .super_resolution_control_half_degree_azimuth()
        {
            6
        } else {
            3
        }) {
        let next_elevation_index =
            (current_elevation_index + 1) % volume_coverage_pattern.elevations.len();
        volume_coverage_pattern.elevations[next_elevation_index].waveform_type()
    } else {
        volume_coverage_pattern.elevations[current_elevation_index].waveform_type()
    };

    // Contiguous Surveillance: ~4 seconds, other waveform types: ~11 seconds
    let estimated_wait_time = if format!("{:?}", next_chunk_waveform) == "CS" {
        Duration::from_secs(4)
    } else {
        Duration::from_secs(11)
    };

    let previous_time = previous_chunk.date_time().unwrap_or_else(Utc::now);
    previous_time.add(Duration::from_millis(estimated_wait_time.as_millis() as u64))
}
