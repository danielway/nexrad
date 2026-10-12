use crate::aws::realtime::{
    ChunkCharacteristics, ChunkIdentifier, ChunkTimingModel, ChunkTimingStats, ElevationChunkMapper,
};
use chrono::{DateTime, Duration, Utc};
use nexrad_decode::messages::volume_coverage_pattern;

/// A projected timeline for all remaining chunks in a volume scan.
///
/// Built from the VCP's azimuth rates and elevation angles using a physics-based model,
/// optionally refined with historical timing observations. This enables UIs to show
/// projected timelines for the entire remaining scan, not just the next chunk.
#[cfg_attr(not(test), allow(dead_code))] // Fully exercised only by the unit tests.
#[derive(Debug, Clone)]
pub(crate) struct ScanTimingProjection {
    /// The sequence number of the anchor chunk (the last observed chunk).
    anchor_sequence: usize,
    /// Projected timing for each future chunk, in sequence order.
    chunks: Vec<ChunkProjection>,
    /// Projected time when the final chunk becomes available.
    volume_end_time: DateTime<Utc>,
    /// Projected total remaining duration from anchor to volume end.
    remaining_duration: Duration,
}

#[cfg_attr(not(test), allow(dead_code))] // Fully exercised only by the unit tests.
impl ScanTimingProjection {
    /// The sequence number of the anchor chunk this projection is relative to.
    pub(crate) fn anchor_sequence(&self) -> usize {
        self.anchor_sequence
    }

    /// Projected timing for each future chunk, in sequence order.
    pub(crate) fn chunks(&self) -> &[ChunkProjection] {
        &self.chunks
    }

    /// Projected time when the final chunk becomes available.
    pub(crate) fn volume_end_time(&self) -> DateTime<Utc> {
        self.volume_end_time
    }

    /// Projected remaining duration from anchor to volume end.
    pub(crate) fn remaining_duration(&self) -> Duration {
        self.remaining_duration
    }
}

/// Projection for a single future chunk.
#[cfg_attr(not(test), allow(dead_code))] // Fully exercised only by the unit tests.
#[derive(Debug, Clone)]
pub(crate) struct ChunkProjection {
    /// The chunk's sequence number.
    sequence: usize,
    /// Projected time this chunk becomes available in S3.
    projected_time: DateTime<Utc>,
    /// Whether this chunk starts a new sweep (useful for UI grouping).
    starts_new_sweep: bool,
}

#[cfg_attr(not(test), allow(dead_code))] // Fully exercised only by the unit tests.
impl ChunkProjection {
    /// The chunk's sequence number.
    pub(crate) fn sequence(&self) -> usize {
        self.sequence
    }

    /// Projected time this chunk becomes available in S3.
    pub(crate) fn projected_time(&self) -> DateTime<Utc> {
        self.projected_time
    }

    /// Whether this chunk starts a new sweep.
    pub(crate) fn starts_new_sweep(&self) -> bool {
        self.starts_new_sweep
    }
}

/// Build a timing projection for all remaining chunks in the current volume.
///
/// The projection starts from `anchor_chunk` (the most recently observed chunk) and
/// projects forward through the final chunk in the volume. Each chunk's projected
/// availability time is computed using the VCP's azimuth rates, elevation angles,
/// and optionally historical timing data.
///
/// Returns `None` if the anchor chunk's metadata cannot be resolved or if there are
/// no remaining chunks to project.
pub(crate) fn project_scan_timing(
    anchor_chunk: &ChunkIdentifier,
    _vcp: &volume_coverage_pattern::Message,
    mapper: &ElevationChunkMapper,
    timing_stats: Option<&ChunkTimingStats>,
) -> Option<ScanTimingProjection> {
    let anchor_sequence = anchor_chunk.sequence();
    let anchor_time = anchor_chunk.upload_date_time().unwrap_or_else(Utc::now);
    let final_sequence = mapper.final_sequence();

    // Nothing to project if we're at or past the final chunk
    if anchor_sequence >= final_sequence {
        return None;
    }

    let anchor_metadata = mapper.get_chunk_metadata(anchor_sequence)?;

    let mut projections = Vec::new();
    let mut cumulative_offset_ms: i64 = 0;
    let mut prev_metadata = anchor_metadata;

    for seq in (anchor_sequence + 1)..=final_sequence {
        let next_metadata = mapper.get_chunk_metadata(seq)?;

        // Compute interval using physics model
        let mut interval_secs =
            ChunkTimingModel::estimate_chunk_interval_secs(prev_metadata, next_metadata);

        // Blend with historical data if available
        if let Some(stats) = timing_stats {
            if let Some(elev_num) = next_metadata.elevation_number() {
                if let Some(elev_data) = _vcp.elevations().get(elev_num - 1) {
                    let characteristics = ChunkCharacteristics {
                        chunk_type: crate::aws::realtime::ChunkType::Intermediate,
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

        let interval_ms = (interval_secs * 1000.0) as i64;
        cumulative_offset_ms += interval_ms;

        let offset_duration = Duration::milliseconds(cumulative_offset_ms);
        let projected_time = anchor_time + offset_duration;

        projections.push(ChunkProjection {
            sequence: seq,
            projected_time,
            starts_new_sweep: next_metadata.is_first_in_sweep(),
        });

        prev_metadata = next_metadata;
    }

    let volume_end_time = projections
        .last()
        .map(|p| p.projected_time)
        .unwrap_or(anchor_time);
    let remaining_duration = Duration::milliseconds(cumulative_offset_ms);

    Some(ScanTimingProjection {
        anchor_sequence,
        chunks: projections,
        volume_end_time,
        remaining_duration,
    })
}
