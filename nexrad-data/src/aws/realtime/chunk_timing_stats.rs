use crate::aws::realtime::{ChunkType, ElevationChunkMapper};
use chrono::Duration;
use nexrad_decode::messages::volume_coverage_pattern::{self, ChannelConfiguration, WaveformType};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

/// Maximum number of timing samples to keep per chunk characteristics
const MAX_TIMING_SAMPLES: usize = 10;

/// Characteristics of a chunk that affect timing.
///
/// Timing samples are keyed by the characteristics of the chunk they lead into: the interval
/// between the uploads of chunks N-1 and N is recorded under chunk N's characteristics when N
/// arrives, and read back under the same characteristics to predict N from N-1.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ChunkCharacteristics {
    /// Type of the chunk
    pub(crate) chunk_type: ChunkType,
    /// Waveform type of the elevation
    pub(crate) waveform_type: WaveformType,
    /// Channel configuration of the elevation
    pub(crate) channel_configuration: ChannelConfiguration,
    /// Whether the chunk is the first in its sweep, so the interval into it includes the
    /// inter-sweep transition
    pub(crate) is_first_in_sweep: bool,
}

impl ChunkCharacteristics {
    /// Returns the characteristics of the chunk at `sequence` with the given type, as described by
    /// the VCP and its chunk mapping. Returns `None` for the Start chunk, which belongs to no
    /// elevation, and for a sequence outside the volume.
    pub(crate) fn for_chunk(
        sequence: usize,
        chunk_type: ChunkType,
        vcp: &volume_coverage_pattern::Message,
        mapper: &ElevationChunkMapper,
    ) -> Option<Self> {
        let metadata = mapper.get_chunk_metadata(sequence)?;
        let elevation = vcp
            .elevations()
            .get(metadata.elevation_number()?.checked_sub(1)?)?;

        Some(Self {
            chunk_type,
            waveform_type: elevation.waveform_type(),
            channel_configuration: elevation.channel_configuration(),
            is_first_in_sweep: metadata.is_first_in_sweep(),
        })
    }
}

impl Hash for ChunkCharacteristics {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(&self.chunk_type).hash(state);
        std::mem::discriminant(&self.waveform_type).hash(state);
        std::mem::discriminant(&self.channel_configuration).hash(state);
        self.is_first_in_sweep.hash(state);
    }
}

/// Statistics for a single timing sample
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimingStat {
    /// Duration of the timing sample
    duration: Duration,
    /// Number of attempts to download the chunk
    attempts: usize,
}

/// Statistics for timing between chunks
#[derive(Debug, Clone, Default)]
pub(crate) struct ChunkTimingStats {
    /// Timing statistics for each chunk characteristics
    timings: HashMap<ChunkCharacteristics, VecDeque<TimingStat>>,
}

impl ChunkTimingStats {
    /// Create a new empty timing statistics
    pub(crate) fn new() -> Self {
        Self {
            timings: HashMap::new(),
        }
    }

    /// Add a timing sample for the given chunk characteristics
    pub(crate) fn add_timing(
        &mut self,
        characteristics: ChunkCharacteristics,
        duration: Duration,
        attempts: usize,
    ) {
        let entry = self.timings.entry(characteristics).or_default();

        entry.push_back(TimingStat { duration, attempts });

        // Maintain the rolling window by removing oldest if we exceed the max
        if entry.len() > MAX_TIMING_SAMPLES {
            entry.pop_front();
        }
    }

    /// Get the average timing for the given chunk characteristics
    pub(crate) fn get_average_timing(
        &self,
        characteristics: &ChunkCharacteristics,
    ) -> Option<Duration> {
        self.timings.get(characteristics).and_then(|timings| {
            if timings.is_empty() {
                return None;
            }

            let total_millis: i64 = timings
                .iter()
                .map(|timing| timing.duration.num_milliseconds())
                .sum();

            let avg_millis = total_millis / timings.len() as i64;
            Some(Duration::milliseconds(avg_millis))
        })
    }

    /// Get the average number of attempts for the given chunk characteristics
    pub(crate) fn get_average_attempts(
        &self,
        characteristics: &ChunkCharacteristics,
    ) -> Option<f64> {
        self.timings.get(characteristics).and_then(|timings| {
            if timings.is_empty() {
                return None;
            }

            let total_attempts: usize = timings.iter().map(|timing| timing.attempts).sum();
            Some(total_attempts as f64 / timings.len() as f64)
        })
    }
}
