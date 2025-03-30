use crate::aws::realtime::{
    ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats, ChunkType, ElevationChunkMapper,
};
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Utc};
use log::debug;
use std::ops::Add;

/// Attempts to estimate the time at which the next chunk will be available given the previous
/// chunk. Requires an [ElevationChunkMapper] to describe the relationship between chunk sequence
/// and VCP elevations. A None result indicates that the chunk is already available or that an
/// estimate cannot be made.
pub fn estimate_next_chunk_time(
    previous_chunk: &ChunkIdentifier,
    elevation_chunk_mapper: &ElevationChunkMapper,
    timing_stats: Option<&ChunkTimingStats>,
) -> Option<DateTime<Utc>> {
    if previous_chunk.chunk_type() == Some(ChunkType::End) {
        return Some(
            previous_chunk
                .date_time()
                .unwrap_or_else(Utc::now)
                .add(ChronoDuration::seconds(10)),
        );
    }

    if let Some(previous_sequence) = previous_chunk.sequence() {
        // Get the next sequence and corresponding elevation
        let next_sequence = previous_sequence + 1;

        if let Some(elevation) = elevation_chunk_mapper.get_sequence_elevation(next_sequence) {
            let next_chunk_type = elevation_chunk_mapper.get_sequence_type(next_sequence);

            let waveform_type = elevation.waveform_type();
            let channel_config = elevation.channel_configuration();

            let characteristics = ChunkCharacteristics {
                chunk_type: next_chunk_type,
                waveform_type,
                channel_configuration: channel_config,
            };

            let average_timing =
                timing_stats.and_then(|stats| stats.get_average_timing(&characteristics));
            let average_attempts =
                timing_stats.and_then(|stats| stats.get_average_attempts(&characteristics));

            // Check if we have historical timing data for this combination
            let estimated_wait_time = if let (Some(avg_timing), Some(avg_attempts)) =
                (average_timing, average_attempts)
            {
                // Use historical average if available
                let mut wait_time = avg_timing;

                // If we're making multiple attempts, add the average number of attempts to the wait time
                wait_time += chrono::Duration::seconds(avg_attempts as i64 - 1);

                debug!(
                    "Using historical average timing of {}ms and {} attempts for {}ms",
                    avg_timing.num_milliseconds(),
                    avg_attempts,
                    wait_time.num_milliseconds()
                );

                wait_time
            } else {
                // Fall back to the static estimation
                let wait_time = get_default_wait_time(waveform_type, channel_config);

                debug!(
                    "No historical timing data available, using static estimation of {}ms",
                    wait_time.num_milliseconds()
                );

                wait_time
            };

            let previous_time = previous_chunk.date_time().unwrap_or_else(Utc::now);
            return Some(previous_time.add(estimated_wait_time));
        };
    }

    None
}

/// Gets the default wait time based on waveform type and channel configuration
fn get_default_wait_time(
    waveform_type: nexrad_decode::messages::volume_coverage_pattern::WaveformType,
    channel_config: nexrad_decode::messages::volume_coverage_pattern::ChannelConfiguration,
) -> ChronoDuration {
    use nexrad_decode::messages::volume_coverage_pattern::{ChannelConfiguration, WaveformType};

    if waveform_type == WaveformType::CS {
        ChronoDuration::seconds(11)
    } else if channel_config == ChannelConfiguration::ConstantPhase {
        ChronoDuration::seconds(7)
    } else {
        ChronoDuration::seconds(4)
    }
}
