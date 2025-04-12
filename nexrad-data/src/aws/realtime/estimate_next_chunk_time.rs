use crate::aws::realtime::{
    ChunkCharacteristics, ChunkIdentifier, ChunkTimingStats, ChunkType, ElevationChunkMapper,
};
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Utc};
use log::debug;
use nexrad_decode::messages::volume_coverage_pattern::{self, ChannelConfiguration, WaveformType};

/// Attempts to estimate the time at which the next chunk will be available given the previous
/// chunk. Requires an [ElevationChunkMapper] to describe the relationship between chunk sequence
/// and VCP elevations. A None result indicates that the chunk is already available or that an
/// estimate cannot be made.
pub fn estimate_chunk_availability_time(
    chunk: &ChunkIdentifier,
    vcp: &volume_coverage_pattern::Message,
    elevation_chunk_mapper: &ElevationChunkMapper,
    timing_stats: Option<&ChunkTimingStats>,
) -> Option<DateTime<Utc>> {
    let processing_time =
        estimate_chunk_processing_time(chunk, vcp, elevation_chunk_mapper, timing_stats)?;

    let now = Utc::now();
    let availability_time = now + processing_time;

    Some(availability_time)
}

/// Attempts to estimate the time the given chunk will take to become available in the real-time S3
/// bucket following the previous chunk. Requires an [ElevationChunkMapper] to describe the
/// relationship between chunk sequence and VCP elevations. A None result indicates that an estimate
/// cannot be made.
pub fn estimate_chunk_processing_time(
    chunk: &ChunkIdentifier,
    vcp: &volume_coverage_pattern::Message,
    elevation_chunk_mapper: &ElevationChunkMapper,
    timing_stats: Option<&ChunkTimingStats>,
) -> Option<ChronoDuration> {
    if chunk.chunk_type() == Some(ChunkType::Start) {
        return Some(ChronoDuration::seconds(10));
    }

    if let (Some(sequence), Some(chunk_type)) = (chunk.sequence(), chunk.chunk_type()) {
        if let Some(elevation) = elevation_chunk_mapper
            .get_sequence_elevation_number(sequence)
            .and_then(|elevation_number| vcp.elevations.get(elevation_number - 1))
        {
            let waveform_type = elevation.waveform_type();
            let channel_config = elevation.channel_configuration();

            let characteristics = ChunkCharacteristics {
                chunk_type,
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

            return Some(estimated_wait_time);
        }
    }

    None
}

/// Gets the default wait time based on waveform type and channel configuration
fn get_default_wait_time(
    waveform_type: WaveformType,
    channel_config: ChannelConfiguration,
) -> ChronoDuration {
    if waveform_type == WaveformType::CS {
        ChronoDuration::seconds(11)
    } else if channel_config == ChannelConfiguration::ConstantPhase {
        ChronoDuration::seconds(7)
    } else {
        ChronoDuration::seconds(4)
    }
}
