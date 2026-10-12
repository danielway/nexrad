use crate::aws::realtime::{
    ChunkCharacteristics, ChunkIdentifier, ChunkTimingModel, ChunkTimingStats, ChunkType,
    ElevationChunkMapper,
};
use chrono::{DateTime, Duration, Utc};
use nexrad_decode::messages::volume_coverage_pattern;

/// Projects the time at which the final chunk of `anchor_chunk`'s volume becomes available for
/// download from S3.
///
/// The projection starts from `anchor_chunk` (the most recently observed chunk) and sums the
/// expected interval to each remaining chunk, computed from the VCP's azimuth rates and elevation
/// angles and, when available, blended with historical timing data.
///
/// Returns `None` if the anchor chunk's metadata cannot be resolved or if there are no remaining
/// chunks to project.
pub(crate) fn project_volume_end_time(
    anchor_chunk: &ChunkIdentifier,
    vcp: &volume_coverage_pattern::Message,
    mapper: &ElevationChunkMapper,
    timing_stats: Option<&ChunkTimingStats>,
) -> Option<DateTime<Utc>> {
    let anchor_sequence = anchor_chunk.sequence();
    let anchor_time = anchor_chunk.upload_date_time().unwrap_or_else(Utc::now);
    let final_sequence = mapper.final_sequence();

    // Nothing to project if we're at or past the final chunk
    if anchor_sequence >= final_sequence {
        return None;
    }

    let mut prev_metadata = mapper.get_chunk_metadata(anchor_sequence)?;
    let mut cumulative_offset_ms: i64 = 0;

    for seq in (anchor_sequence + 1)..=final_sequence {
        let next_metadata = mapper.get_chunk_metadata(seq)?;

        // Compute interval using physics model
        let mut interval_secs =
            ChunkTimingModel::estimate_chunk_interval_secs(prev_metadata, next_metadata);

        // Blend with historical data if available
        if let Some(stats) = timing_stats {
            if let Some(elev_num) = next_metadata.elevation_number() {
                if let Some(elev_data) = vcp.elevations().get(elev_num - 1) {
                    let characteristics = ChunkCharacteristics {
                        chunk_type: ChunkType::Intermediate,
                        waveform_type: elev_data.waveform_type(),
                        channel_configuration: elev_data.channel_configuration(),
                    };

                    if let Some(avg_timing) = stats.get_average_timing(&characteristics) {
                        let historical_secs = avg_timing.num_milliseconds() as f64 / 1000.0;
                        // Blend: 70% physics, 30% historical
                        interval_secs = interval_secs * 0.7 + historical_secs * 0.3;
                    }
                }
            }
        }

        cumulative_offset_ms += (interval_secs * 1000.0) as i64;
        prev_metadata = next_metadata;
    }

    Some(anchor_time + Duration::milliseconds(cumulative_offset_ms))
}
