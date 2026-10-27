//! Unit tests for the crate-private real-time timing, projection, and chunk-sequencing
//! internals. Tests of the public real-time surface live in `nexrad-data/tests/`.

use super::{
    next_volume_start, Chunk, ChunkCharacteristics, ChunkIdentifier, ChunkTimingModel,
    ChunkTimingStats, ChunkType, ElevationChunkMapper, NextChunk, VolumeIndex,
};
use chrono::{Duration, NaiveDateTime};
use nexrad_decode::messages::volume_coverage_pattern::{self, ChannelConfiguration, WaveformType};
use std::sync::OnceLock;

/// Path to the archive volume used to obtain a real VCP. Read at runtime rather than embedded
/// with `include_bytes!` so the library test binary does not carry the whole file.
const TEST_NEXRAD_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../downloads/KDMX20220305_232324_V06"
);

/// Returns the VCP from the test archive volume, decoding it once per test binary.
pub(super) fn get_test_vcp() -> volume_coverage_pattern::Message<'static> {
    static VCP: OnceLock<volume_coverage_pattern::Message<'static>> = OnceLock::new();
    VCP.get_or_init(|| {
        let data = std::fs::read(TEST_NEXRAD_FILE).expect("read test volume");
        crate::volume::File::new(data)
            .coverage_pattern_message()
            .expect("test volume carries a VCP")
    })
    .clone()
}

#[test]
fn test_elevation_chunk_mapper_creation() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // Mapper should have a valid final sequence
    let final_seq = mapper.final_sequence();
    assert!(final_seq > 0, "Final sequence should be positive");

    // For typical VCPs, final sequence is usually in the 40-80 range
    assert!(
        final_seq < 200,
        "Final sequence seems unreasonably large: {}",
        final_seq
    );
}

#[test]
fn test_elevation_chunk_mapper_sequence_1_is_metadata() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // Sequence 1 is the metadata chunk, not an elevation scan
    let elevation = mapper.get_sequence_elevation_number(1);
    assert_eq!(elevation, None, "Sequence 1 should not map to an elevation");
}

#[test]
fn test_elevation_chunk_mapper_sequence_2_is_first_elevation() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // Sequence 2 should be the first elevation scan
    let elevation = mapper.get_sequence_elevation_number(2);
    assert_eq!(
        elevation,
        Some(1),
        "Sequence 2 should map to first elevation"
    );
}

#[test]
fn test_elevation_chunk_mapper_all_sequences_valid() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let final_seq = mapper.final_sequence();

    // All sequences from 2 to final should map to some elevation
    for seq in 2..=final_seq {
        let elevation = mapper.get_sequence_elevation_number(seq);
        assert!(
            elevation.is_some(),
            "Sequence {} should map to an elevation",
            seq
        );

        let elev_num = elevation.unwrap();
        assert!(
            elev_num >= 1 && elev_num <= vcp.elevations().len(),
            "Elevation number {} out of range for sequence {}",
            elev_num,
            seq
        );
    }
}

#[test]
fn test_elevation_chunk_mapper_final_sequence() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let final_seq = mapper.final_sequence();

    // Final sequence should map to last elevation
    let elevation = mapper.get_sequence_elevation_number(final_seq);
    assert!(
        elevation.is_some(),
        "Final sequence should map to an elevation"
    );
    assert_eq!(
        elevation.unwrap(),
        vcp.elevations().len(),
        "Final sequence should map to last elevation"
    );
}

#[test]
fn test_elevation_chunk_mapper_beyond_final_sequence() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let final_seq = mapper.final_sequence();

    // Sequences beyond final should return None
    let elevation = mapper.get_sequence_elevation_number(final_seq + 1);
    assert_eq!(
        elevation, None,
        "Sequence beyond final should not map to elevation"
    );

    let elevation = mapper.get_sequence_elevation_number(final_seq + 100);
    assert_eq!(
        elevation, None,
        "Far beyond final sequence should not map to elevation"
    );
}

// === ChunkMetadata tests ===

#[test]
fn test_chunk_metadata_sequence_1_is_start() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let meta = mapper.get_chunk_metadata(1).expect("sequence 1 metadata");
    assert!(meta.is_start_chunk());
    assert_eq!(meta.elevation_number(), None);
}

#[test]
fn test_chunk_metadata_first_elevation() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let meta = mapper.get_chunk_metadata(2).expect("sequence 2 metadata");
    assert!(!meta.is_start_chunk());
    assert_eq!(meta.elevation_number(), Some(1));
    assert!(meta.is_first_in_sweep());
    assert!(meta.azimuth_rate_dps() > 0.0);

    // First elevation of VCP 212 is super-res (0.5 deg azimuth), so 6 chunks
    let first_elev = &vcp.elevations()[0];
    if first_elev.super_resolution_half_degree_azimuth() {
        assert_eq!(meta.chunks_in_sweep(), 6);
    } else {
        assert_eq!(meta.chunks_in_sweep(), 3);
    }
}

#[test]
fn test_chunk_metadata_sweep_boundaries() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // Skip the Start chunk (sequence 1); each change of elevation must start a new sweep and no
    // other chunk may.
    let mut current_elev = None;
    for seq in 2..=mapper.final_sequence() {
        let meta = mapper.get_chunk_metadata(seq).expect("metadata in range");
        let new_sweep = meta.elevation_number() != current_elev;
        assert_eq!(
            meta.is_first_in_sweep(),
            new_sweep,
            "Chunk seq {} first-in-sweep mismatch for elev {:?}",
            seq,
            meta.elevation_number()
        );
        current_elev = meta.elevation_number();
    }
}

#[test]
fn test_chunk_metadata_out_of_range() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    assert!(mapper.get_chunk_metadata(0).is_none());
    assert!(mapper.get_chunk_metadata(mapper.final_sequence()).is_some());
    assert!(mapper
        .get_chunk_metadata(mapper.final_sequence() + 1)
        .is_none());
}

#[test]
fn test_chunk_metadata_azimuth_rates_match_vcp() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // For each elevation, all chunks should have the same azimuth rate from the VCP
    for (elev_idx, elev_data) in vcp.elevations().iter().enumerate() {
        let elev_num = elev_idx + 1;
        let expected_rate = elev_data.azimuth_rate_degrees_per_second();

        for seq in 1..=mapper.final_sequence() {
            let meta = mapper.get_chunk_metadata(seq).expect("metadata in range");
            if meta.elevation_number() == Some(elev_num) {
                assert!(
                    (meta.azimuth_rate_dps() - expected_rate).abs() < 0.001,
                    "Elevation {} azimuth rate mismatch: meta={}, vcp={}",
                    elev_num,
                    meta.azimuth_rate_dps(),
                    expected_rate
                );
            }
        }
    }
}

// === ChunkCharacteristics tests ===

#[test]
fn test_chunk_characteristics_equality() {
    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    let char3 = ChunkCharacteristics {
        chunk_type: ChunkType::End,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    assert_eq!(char1, char2);
    assert_ne!(char1, char3);
}

#[test]
fn test_chunk_characteristics_hash() {
    use std::collections::HashSet;

    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    let mut set = HashSet::new();
    set.insert(char1);
    set.insert(char2);

    // char1 and char2 are equal, so set should have only 1 element
    assert_eq!(set.len(), 1);
}

#[test]
fn test_chunk_characteristics_distinguish_sweep_position() {
    use std::collections::HashSet;

    let first = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: true,
    };
    let intra = ChunkCharacteristics {
        is_first_in_sweep: false,
        ..first
    };

    assert_ne!(first, intra);

    let set: HashSet<_> = [first, intra].into_iter().collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn test_chunk_characteristics_for_chunk() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // The Start chunk belongs to no elevation
    assert!(ChunkCharacteristics::for_chunk(1, ChunkType::Start, &vcp, &mapper).is_none());
    assert!(ChunkCharacteristics::for_chunk(
        mapper.final_sequence() + 1,
        ChunkType::Intermediate,
        &vcp,
        &mapper
    )
    .is_none());

    // Sequences 2 and 3 share the first elevation and differ only in sweep position
    let first = ChunkCharacteristics::for_chunk(2, ChunkType::Intermediate, &vcp, &mapper).unwrap();
    let second =
        ChunkCharacteristics::for_chunk(3, ChunkType::Intermediate, &vcp, &mapper).unwrap();
    assert!(first.is_first_in_sweep);
    assert!(!second.is_first_in_sweep);
    assert_eq!(first.waveform_type, second.waveform_type);
    assert_eq!(first.channel_configuration, second.channel_configuration);
    assert_ne!(first, second);
}

// === ChunkTimingStats tests ===

#[test]
fn test_chunk_timing_stats_new() {
    let stats = ChunkTimingStats::new();
    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    // New stats should have no samples
    assert_eq!(stats.get_average_timing(&characteristics), None);
    assert_eq!(stats.get_average_attempts(&characteristics), None);
}

#[test]
fn test_chunk_timing_stats_add_and_retrieve() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    // Add a timing sample
    let duration = Duration::seconds(5);
    stats.add_timing(characteristics, duration, 1);

    assert_eq!(
        stats.get_average_timing(&characteristics),
        Some(Duration::seconds(5))
    );
    assert_eq!(stats.get_average_attempts(&characteristics), Some(1.0));
}

#[test]
fn test_chunk_timing_stats_average() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    // Add multiple samples
    stats.add_timing(characteristics, Duration::seconds(4), 1);
    stats.add_timing(characteristics, Duration::seconds(6), 1);
    stats.add_timing(characteristics, Duration::seconds(5), 1);

    // Average should be 5 seconds
    assert_eq!(
        stats.get_average_timing(&characteristics),
        Some(Duration::seconds(5))
    );
    // Average attempts should be 1.0
    assert_eq!(stats.get_average_attempts(&characteristics), Some(1.0));
}

#[test]
fn test_chunk_timing_stats_different_characteristics() {
    let mut stats = ChunkTimingStats::new();

    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::End,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    stats.add_timing(char1, Duration::seconds(5), 1);
    stats.add_timing(char2, Duration::seconds(10), 2);

    // Should track them separately
    assert_eq!(stats.get_average_timing(&char1), Some(Duration::seconds(5)));
    assert_eq!(stats.get_average_attempts(&char1), Some(1.0));
    assert_eq!(
        stats.get_average_timing(&char2),
        Some(Duration::seconds(10))
    );
    assert_eq!(stats.get_average_attempts(&char2), Some(2.0));
}

#[test]
fn test_chunk_timing_stats_separate_sweep_position_buckets() {
    let mut stats = ChunkTimingStats::new();

    let first = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: true,
    };
    let intra = ChunkCharacteristics {
        is_first_in_sweep: false,
        ..first
    };

    stats.add_timing(first, Duration::seconds(9), 1);
    stats.add_timing(intra, Duration::seconds(3), 1);

    assert_eq!(stats.get_average_timing(&first), Some(Duration::seconds(9)));
    assert_eq!(stats.get_average_timing(&intra), Some(Duration::seconds(3)));
}

#[test]
fn test_chunk_timing_stats_rolling_window() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    // Add more than MAX_TIMING_SAMPLES (which is 10)
    for i in 1..=15 {
        stats.add_timing(characteristics, Duration::seconds(i), 1);
    }

    // Should only keep the last 10 samples. Average of last 10 samples (6-15) is 10.5 seconds:
    // (6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15) / 10 = 105 / 10 = 10.5
    assert_eq!(
        stats.get_average_timing(&characteristics),
        Some(Duration::milliseconds(10500))
    );
}

#[test]
fn test_chunk_timing_stats_attempts_tracking() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    // Add samples with different attempt counts
    stats.add_timing(characteristics, Duration::seconds(5), 1);
    stats.add_timing(characteristics, Duration::seconds(5), 2);
    stats.add_timing(characteristics, Duration::seconds(5), 3);

    // Average attempts should be 2.0
    assert_eq!(stats.get_average_attempts(&characteristics), Some(2.0));
}

#[test]
fn test_chunk_timing_stats_clone() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::ContiguousSurveillance,
        channel_configuration: ChannelConfiguration::ConstantPhase,
        is_first_in_sweep: false,
    };

    stats.add_timing(characteristics, Duration::seconds(5), 1);

    let cloned = stats.clone();
    assert_eq!(
        cloned.get_average_timing(&characteristics),
        Some(Duration::seconds(5))
    );
}

// === Timing estimation tests ===

fn chunk_at(sequence: usize, chunk_type: ChunkType) -> ChunkIdentifier {
    ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1).unwrap(),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        sequence,
        chunk_type,
        None,
    )
}

#[test]
fn test_estimate_chunk_processing_time_start_chunk() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let estimate =
        estimate_chunk_processing_time(&chunk_at(1, ChunkType::Start), &vcp, &mapper, None);

    // The first radial chunk follows the Start chunk after 1.5 seconds
    assert_eq!(estimate, Some(Duration::milliseconds(1500)));
}

#[test]
fn test_estimate_chunk_processing_time_start_chunk_ignores_stats() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let mut stats = ChunkTimingStats::new();
    let second =
        ChunkCharacteristics::for_chunk(2, ChunkType::Intermediate, &vcp, &mapper).unwrap();
    stats.add_timing(second, Duration::seconds(20), 1);

    let estimate =
        estimate_chunk_processing_time(&chunk_at(1, ChunkType::Start), &vcp, &mapper, Some(&stats));
    assert_eq!(estimate, Some(Duration::milliseconds(1500)));
}

#[test]
fn test_end_to_start_interval_is_inter_volume_gap() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let end = mapper.get_chunk_metadata(mapper.final_sequence()).unwrap();
    let start = mapper.get_chunk_metadata(1).unwrap();

    let interval = ChunkTimingModel::estimate_chunk_interval_secs(end, start);
    assert!((interval - ChunkTimingModel::inter_volume_gap_secs()).abs() < 0.001);
    assert!((interval - ChunkTimingModel::start_to_first_intermediate_gap_secs()).abs() > 1.0);
}

#[test]
fn test_estimate_chunk_processing_time_end_chunk_is_inter_volume_gap() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let end = chunk_at(mapper.final_sequence(), ChunkType::End);
    let estimate = estimate_chunk_processing_time(&end, &vcp, &mapper, None);
    let gap_ms = (ChunkTimingModel::inter_volume_gap_secs() * 1000.0) as i64;
    assert_eq!(estimate, Some(Duration::milliseconds(gap_ms)));
}

#[test]
fn test_estimate_chunk_processing_time_with_stats() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // A sample recorded on the arrival of sequence 3 predicts sequence 3 from sequence 2
    let characteristics =
        ChunkCharacteristics::for_chunk(3, ChunkType::Intermediate, &vcp, &mapper).unwrap();
    let mut stats = ChunkTimingStats::new();
    stats.add_timing(characteristics, Duration::seconds(5), 2);

    let estimate = estimate_chunk_processing_time(
        &chunk_at(2, ChunkType::Intermediate),
        &vcp,
        &mapper,
        Some(&stats),
    );

    // Historical timing (5 seconds) plus 1 second for the extra attempt
    assert_eq!(estimate, Some(Duration::seconds(6)));
}

#[test]
fn test_estimate_chunk_processing_time_keys_stats_by_predicted_chunk_across_sweeps() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // The first chunk of the second sweep, predicted from the last chunk of the first sweep
    let next_sequence = (2..=mapper.final_sequence())
        .find(|&sequence| {
            sequence > 2
                && mapper
                    .get_chunk_metadata(sequence)
                    .unwrap()
                    .is_first_in_sweep()
        })
        .unwrap();
    let previous = chunk_at(next_sequence - 1, ChunkType::Intermediate);

    let physics = estimate_chunk_processing_time(&previous, &vcp, &mapper, None).unwrap();

    // A sample under the previous chunk's characteristics does not predict the next chunk
    let mut stats = ChunkTimingStats::new();
    let previous_characteristics =
        ChunkCharacteristics::for_chunk(next_sequence - 1, ChunkType::Intermediate, &vcp, &mapper)
            .unwrap();
    stats.add_timing(previous_characteristics, Duration::seconds(40), 1);
    assert_eq!(
        estimate_chunk_processing_time(&previous, &vcp, &mapper, Some(&stats)),
        Some(physics)
    );

    // A sample under the next chunk's characteristics does
    let next_characteristics =
        ChunkCharacteristics::for_chunk(next_sequence, ChunkType::Intermediate, &vcp, &mapper)
            .unwrap();
    assert!(next_characteristics.is_first_in_sweep);
    stats.add_timing(next_characteristics, Duration::seconds(30), 1);
    assert_eq!(
        estimate_chunk_processing_time(&previous, &vcp, &mapper, Some(&stats)),
        Some(Duration::seconds(30))
    );
}

#[test]
fn test_estimate_chunk_processing_time_predicts_end_chunk_with_end_stats() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);
    let final_sequence = mapper.final_sequence();

    let mut stats = ChunkTimingStats::new();
    let end =
        ChunkCharacteristics::for_chunk(final_sequence, ChunkType::End, &vcp, &mapper).unwrap();
    stats.add_timing(end, Duration::seconds(12), 1);

    let estimate = estimate_chunk_processing_time(
        &chunk_at(final_sequence - 1, ChunkType::Intermediate),
        &vcp,
        &mapper,
        Some(&stats),
    );
    assert_eq!(estimate, Some(Duration::seconds(12)));
}

#[test]
fn test_estimate_chunk_processing_time_without_stats_uses_physics_model() {
    use super::estimate_next_chunk_time::estimate_chunk_processing_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let chunk = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1).unwrap(),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        2,
        ChunkType::Intermediate,
        None,
    );

    let estimate = estimate_chunk_processing_time(&chunk, &vcp, &mapper, None);

    assert!(estimate.is_some());
    let duration_secs = estimate.unwrap().num_milliseconds() as f64 / 1000.0;

    // Physics model: chunk duration = (360 / azimuth_rate - 0.67) / chunks_in_sweep
    // For VCP 212 first elevation, azimuth rate ~21 dps, super-res (6 chunks)
    // Expected: (360/21.149 - 0.67) / 6 ≈ 2.73s
    // With inter-sweep gap for first-in-sweep: ~2.73 + 0.7 = ~3.43s
    // The exact value depends on whether this is first-in-sweep or intra-sweep
    assert!(
        duration_secs > 1.0 && duration_secs < 15.0,
        "Physics-model estimate should be reasonable, got: {:.2}s",
        duration_secs
    );
}

// === ChunkTimingModel tests ===

#[test]
fn test_chunk_timing_model_sweep_duration_precip() {
    // VCP 212 low elevation: ~21.149 dps
    let duration = ChunkTimingModel::sweep_duration_secs(21.149).unwrap();
    // Expected: (360 / 21.149) - 0.67 ≈ 16.36s
    assert!(
        (duration - 16.36).abs() < 0.1,
        "Expected ~16.36s, got {:.2}s",
        duration
    );
}

#[test]
fn test_chunk_timing_model_sweep_duration_clear_air() {
    // VCP 35 low elevation: ~4.966 dps
    let duration = ChunkTimingModel::sweep_duration_secs(4.966).unwrap();
    // Expected: (360 / 4.966) - 0.67 ≈ 71.82s
    assert!(
        (duration - 71.82).abs() < 0.1,
        "Expected ~71.82s, got {:.2}s",
        duration
    );
}

#[test]
fn test_chunk_timing_model_sweep_duration_zero_rate() {
    assert!(ChunkTimingModel::sweep_duration_secs(0.0).is_none());
    assert!(ChunkTimingModel::sweep_duration_secs(-5.0).is_none());
}

#[test]
fn test_chunk_timing_model_chunk_duration() {
    // 21.149 dps, 6 chunks (super-res)
    let chunk_dur = ChunkTimingModel::chunk_duration_secs(21.149, 6).unwrap();
    assert!(
        (chunk_dur - 2.73).abs() < 0.1,
        "Expected ~2.73s, got {:.2}s",
        chunk_dur
    );

    // 21.149 dps, 3 chunks (standard)
    let chunk_dur = ChunkTimingModel::chunk_duration_secs(21.149, 3).unwrap();
    assert!(
        (chunk_dur - 5.45).abs() < 0.1,
        "Expected ~5.45s, got {:.2}s",
        chunk_dur
    );

    // Zero chunks
    assert!(ChunkTimingModel::chunk_duration_secs(21.149, 0).is_none());
}

#[test]
fn test_chunk_timing_model_inter_sweep_gap_same_elevation() {
    let gap = ChunkTimingModel::inter_sweep_gap_secs(0.5, 0.5);
    assert!(
        (gap - 0.7).abs() < 0.001,
        "Same elevation gap should be 0.7s"
    );
}

#[test]
fn test_chunk_timing_model_inter_sweep_gap_small_change() {
    let gap = ChunkTimingModel::inter_sweep_gap_secs(0.5, 0.9);
    // 0.7 + (0.4 * 0.08) = 0.732
    assert!(
        (gap - 0.732).abs() < 0.01,
        "Expected ~0.732s, got {:.3}s",
        gap
    );
}

#[test]
fn test_chunk_timing_model_inter_sweep_gap_large_change() {
    let gap = ChunkTimingModel::inter_sweep_gap_secs(6.4, 15.7);
    // 0.7 + (9.3 * 0.08) = 1.444
    assert!(
        (gap - 1.444).abs() < 0.01,
        "Expected ~1.444s, got {:.3}s",
        gap
    );
}

#[test]
fn test_chunk_timing_model_inter_volume_gap() {
    assert!((ChunkTimingModel::inter_volume_gap_secs() - 8.5).abs() < 0.001);
}

// === Volume end time projection tests ===

fn anchor_chunk(sequence: usize, chunk_type: ChunkType) -> ChunkIdentifier {
    use chrono::{TimeZone, Utc};

    ChunkIdentifier::new(
        "KDMX",
        VolumeIndex::new(1).unwrap(),
        NaiveDateTime::parse_from_str("20220305_232324", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        sequence,
        chunk_type,
        Some(Utc.with_ymd_and_hms(2022, 3, 5, 23, 23, 24).unwrap()),
    )
}

#[test]
fn test_project_volume_end_time_after_anchor() {
    use super::project_volume_end_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let anchor = anchor_chunk(5, ChunkType::Intermediate);
    let end_time =
        project_volume_end_time(&anchor, &vcp, &mapper, None).expect("projection should succeed");
    assert!(
        end_time > anchor.upload_date_time().unwrap(),
        "Volume end time should be after the anchor"
    );

    // Projecting from earlier in the volume (same anchor time) reaches the end later
    let from_start = anchor_chunk(1, ChunkType::Start);
    let end_from_start = project_volume_end_time(&from_start, &vcp, &mapper, None).unwrap();
    assert!(end_from_start > end_time);
}

#[test]
fn test_project_volume_end_time_total_duration() {
    use super::project_volume_end_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let anchor = anchor_chunk(1, ChunkType::Start);
    let end_time = project_volume_end_time(&anchor, &vcp, &mapper, None).unwrap();
    let duration_secs = (end_time - anchor.upload_date_time().unwrap()).num_seconds();

    // VCP 212 with SAILS typically takes 200-400 seconds
    assert!(
        duration_secs > 100 && duration_secs < 600,
        "Volume duration should be 100-600s, got {}s",
        duration_secs
    );
}

#[test]
fn test_project_volume_end_time_at_final_sequence() {
    use super::project_volume_end_time;

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // At the final sequence, there's nothing to project
    let anchor = anchor_chunk(mapper.final_sequence(), ChunkType::End);
    assert!(project_volume_end_time(&anchor, &vcp, &mapper, None).is_none());
}

// === ChunkIdentifier::next_chunk tests ===

#[test]
fn test_chunk_identifier_next_chunk_intermediate() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);
    let final_seq = mapper.final_sequence();

    // Test non-final chunk progresses to next sequence
    let chunk_id = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1).unwrap(),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        2,
        ChunkType::Intermediate,
        None,
    );

    match chunk_id.next_chunk(&mapper) {
        Some(NextChunk::Sequence(next)) => {
            assert_eq!(next.sequence(), 3);
            assert_eq!(next.site(), "KDMX");
            assert_eq!(next.volume().as_number(), 1);
        }
        _ => panic!("Expected NextChunk::Sequence for intermediate chunk"),
    }

    // Test final chunk progresses to next volume
    let chunk_id = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1).unwrap(),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        final_seq,
        ChunkType::End,
        None,
    );

    match chunk_id.next_chunk(&mapper) {
        Some(NextChunk::Volume(next_volume)) => {
            assert_eq!(next_volume.as_number(), 2);
        }
        _ => panic!("Expected NextChunk::Volume for final chunk"),
    }
}

// === Volume rollover tests ===

fn volume_chunk(
    volume: usize,
    stamp: &str,
    sequence: usize,
    chunk_type: ChunkType,
) -> ChunkIdentifier {
    ChunkIdentifier::new(
        "KDMX",
        VolumeIndex::new(volume).unwrap(),
        NaiveDateTime::parse_from_str(stamp, "%Y%m%d_%H%M%S")
            .unwrap()
            .and_utc(),
        sequence,
        chunk_type,
        None,
    )
}

/// The listing of a volume whose scan has published sequences `1..=published`.
fn volume_listing(volume: usize, stamp: &str, published: usize) -> Vec<ChunkIdentifier> {
    (1..=published)
        .map(|sequence| {
            let chunk_type = if sequence == 1 {
                ChunkType::Start
            } else {
                ChunkType::Intermediate
            };
            volume_chunk(volume, stamp, sequence, chunk_type)
        })
        .collect()
}

#[test]
fn test_rollover_yields_start_chunk_then_continues_in_sequence() {
    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // The previous volume ended; several chunks of the next volume were published while waiting
    let end = volume_chunk(
        41,
        "20220305_232324",
        mapper.final_sequence(),
        ChunkType::End,
    );
    let next_volume = match end.next_chunk(&mapper) {
        Some(NextChunk::Volume(volume)) => volume,
        other => panic!(
            "Expected NextChunk::Volume after the End chunk, got {:?}",
            other
        ),
    };
    let listed = volume_listing(next_volume.as_number(), "20220305_232852", 7);

    // The Start chunk is fetched rather than the newest listed chunk
    let start = next_volume_start(Some(&end), &listed).expect("next volume has a newer scan");
    assert_eq!(start.volume().as_number(), 42);
    assert_eq!(start.sequence(), 1);
    assert_eq!(start.chunk_type(), ChunkType::Start);
    assert_eq!(start.name(), "20220305-232852-001-S");

    // Iteration then continues from sequence 2 of the new volume's scan
    match start.next_chunk(&mapper) {
        Some(NextChunk::Sequence(next)) => {
            assert_eq!(next.volume().as_number(), 42);
            assert_eq!(next.name(), "20220305-232852-002-I");
        }
        other => panic!(
            "Expected NextChunk::Sequence after the Start chunk, got {:?}",
            other
        ),
    }
}

#[test]
fn test_rollover_targets_start_chunk_of_listed_scan_when_start_not_listed() {
    let end = volume_chunk(41, "20220305_232324", 60, ChunkType::End);
    let listed = vec![
        volume_chunk(42, "20220305_232852", 5, ChunkType::Intermediate),
        volume_chunk(42, "20220305_232852", 6, ChunkType::Intermediate),
    ];

    let start = next_volume_start(Some(&end), &listed).unwrap();
    assert_eq!(start.name(), "20220305-232852-001-S");
}

#[test]
fn test_rollover_waits_for_newer_scan_in_next_volume() {
    let end = volume_chunk(41, "20220305_232324", 60, ChunkType::End);

    // Nothing published yet
    assert!(next_volume_start(Some(&end), &[]).is_none());

    // A reused volume number still holding its previous rotation
    let stale = volume_listing(42, "20220302_101500", 60);
    assert!(next_volume_start(Some(&end), &stale).is_none());
}

#[test]
fn test_rollover_across_volume_number_wrap() {
    let end = volume_chunk(999, "20220305_232324", 60, ChunkType::End);
    let listed = volume_listing(1, "20220305_232852", 3);

    let start = next_volume_start(Some(&end), &listed).unwrap();
    assert_eq!(start.volume().as_number(), 1);
    assert_eq!(start.name(), "20220305-232852-001-S");
}

// === Start chunk VCP tests ===

/// Builds a real-time Start chunk from the test archive volume's header and its LDM record at
/// `record_index`. The archive's first record carries the metadata messages, including the VCP;
/// later records carry only radials.
pub(super) fn start_chunk_from_test_volume(record_index: usize) -> Chunk {
    use crate::volume::{File, Header};

    let data = std::fs::read(TEST_NEXRAD_FILE).expect("read test volume");
    let file = File::new(data);
    let records = file.records().expect("records");

    let mut chunk_data = file.data()[..Header::SIZE].to_vec();
    chunk_data.extend_from_slice(records[record_index].data());
    Chunk::new(chunk_data).expect("start chunk")
}

#[test]
fn test_coverage_pattern_message_from_start_chunk() {
    let Chunk::Start(file) = start_chunk_from_test_volume(0) else {
        panic!("Expected Chunk::Start");
    };

    let vcp = file
        .coverage_pattern_message()
        .expect("start chunk carries a VCP");
    assert_eq!(vcp, get_test_vcp());
    assert!(!vcp.elevations().is_empty());
}

#[test]
fn test_coverage_pattern_message_missing() {
    let Chunk::Start(file) = start_chunk_from_test_volume(1) else {
        panic!("Expected Chunk::Start");
    };

    assert!(matches!(
        file.coverage_pattern_message(),
        Err(crate::result::Error::MissingCoveragePattern)
    ));
}
