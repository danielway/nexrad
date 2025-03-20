use crate::aws::realtime::ChunkIdentifier;
use chrono::{DateTime, Utc};
use std::ops::Add;
use std::time::Duration;

/// Attempts to estimate the time at which the next chunk will be available given the previous chunk.
/// A None result indicates that the chunk is already available or that an estimate cannot be made.
#[cfg(feature = "nexrad-decode")]
pub fn estimate_next_chunk_time(
    previous_chunk: &ChunkIdentifier,
    volume_coverage_pattern: &nexrad_decode::messages::volume_coverage_pattern::Message,
) -> Option<DateTime<Utc>> {
    use super::get_elevation_from_chunk;
    use nexrad_decode::messages::volume_coverage_pattern::{ChannelConfiguration, WaveformType};

    if let Some(previous_sequence) = previous_chunk.sequence() {
        if !((1..55).contains(&previous_sequence)) {
            return None;
        }

        let elevation =
            get_elevation_from_chunk(previous_sequence + 1, &volume_coverage_pattern.elevations);

        if let Some(elevation) = elevation {
            // Estimate based on elevation's waveform and channel configuration
            let estimated_wait_time = if elevation.waveform_type() == WaveformType::CS {
                Duration::from_secs(11)
            } else if elevation.channel_configuration() == ChannelConfiguration::ConstantPhase {
                Duration::from_secs(7)
            } else {
                Duration::from_secs(4)
            };

            let previous_time = previous_chunk.date_time().unwrap_or_else(Utc::now);
            return Some(
                previous_time.add(Duration::from_millis(estimated_wait_time.as_millis() as u64)),
            );
        }
    }

    None
}
