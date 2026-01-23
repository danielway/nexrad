#![cfg(feature = "aws")]

use chrono::Duration;
use nexrad_data::aws::realtime::{
    ChunkCharacteristics, ChunkTimingStats, ChunkType, ElevationChunkMapper,
};
use nexrad_decode::messages::volume_coverage_pattern::{ChannelConfiguration, WaveformType};

// Test data file
const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

// Get a VCP from the test file
fn get_test_vcp() -> nexrad_decode::messages::volume_coverage_pattern::Message<'static> {
    let volume_file = nexrad_data::volume::File::new(TEST_NEXRAD_FILE.to_vec());

    for mut record in volume_file.records().expect("records") {
        if record.compressed() {
            record = record.decompress().unwrap();
        }
        for message in record.messages().unwrap() {
            if let nexrad_decode::messages::MessageContents::VolumeCoveragePattern(vcp) =
                message.contents()
            {
                return vcp.clone().into_owned();
            }
        }
    }
    panic!("No VCP found in test file");
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

#[test]
fn test_chunk_characteristics_equality() {
    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    let char3 = ChunkCharacteristics {
        chunk_type: ChunkType::End,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    assert_eq!(char1, char2);
    assert_ne!(char1, char3);
}

#[test]
fn test_chunk_characteristics_hash() {
    use std::collections::HashSet;

    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    let mut set = HashSet::new();
    set.insert(char1);
    set.insert(char2);

    // char1 and char2 are equal, so set should have only 1 element
    assert_eq!(set.len(), 1);
}

#[test]
fn test_chunk_timing_stats_new() {
    let stats = ChunkTimingStats::new();

    // New stats should be empty
    let all_stats = stats.get_statistics();
    assert_eq!(all_stats.len(), 0, "New stats should be empty");
}

#[test]
fn test_chunk_timing_stats_add_and_retrieve() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    // Add a timing sample
    let duration = Duration::seconds(5);
    stats.add_timing(characteristics, duration, 1);

    // Should be able to retrieve it via get_statistics
    let all_stats = stats.get_statistics();
    assert_eq!(all_stats.len(), 1);

    // Verify the returned stats
    let (char, avg_timing, avg_attempts) = &all_stats[0];
    assert_eq!(char, &characteristics);
    assert_eq!(*avg_timing, Some(Duration::seconds(5)));
    assert_eq!(*avg_attempts, Some(1.0));
}

#[test]
fn test_chunk_timing_stats_average() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    // Add multiple samples
    stats.add_timing(characteristics, Duration::seconds(4), 1);
    stats.add_timing(characteristics, Duration::seconds(6), 1);
    stats.add_timing(characteristics, Duration::seconds(5), 1);

    // Get statistics
    let all_stats = stats.get_statistics();
    assert_eq!(all_stats.len(), 1);

    let (_char, avg_timing, avg_attempts) = &all_stats[0];

    // Average should be 5 seconds
    assert_eq!(*avg_timing, Some(Duration::seconds(5)));
    // Average attempts should be 1.0
    assert_eq!(*avg_attempts, Some(1.0));
}

#[test]
fn test_chunk_timing_stats_different_characteristics() {
    let mut stats = ChunkTimingStats::new();

    let char1 = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    let char2 = ChunkCharacteristics {
        chunk_type: ChunkType::End,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    stats.add_timing(char1, Duration::seconds(5), 1);
    stats.add_timing(char2, Duration::seconds(10), 2);

    // Should track them separately
    let all_stats = stats.get_statistics();
    assert_eq!(all_stats.len(), 2);

    // Find stats for each characteristic
    let mut found_char1 = false;
    let mut found_char2 = false;

    for (char, avg_timing, _avg_attempts) in all_stats {
        if char == char1 {
            assert_eq!(avg_timing, Some(Duration::seconds(5)));
            found_char1 = true;
        } else if char == char2 {
            assert_eq!(avg_timing, Some(Duration::seconds(10)));
            found_char2 = true;
        }
    }

    assert!(
        found_char1 && found_char2,
        "Should find both characteristics"
    );
}

#[test]
fn test_chunk_timing_stats_rolling_window() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    // Add more than MAX_TIMING_SAMPLES (which is 10)
    for i in 1..=15 {
        stats.add_timing(characteristics, Duration::seconds(i), 1);
    }

    // Should only keep the last 10 samples
    let all_stats = stats.get_statistics();
    assert_eq!(all_stats.len(), 1);

    let (_char, avg_timing, _avg_attempts) = &all_stats[0];
    assert!(avg_timing.is_some());

    // Average of last 10 samples (6-15) is 10.5 seconds
    // (6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15) / 10 = 105 / 10 = 10.5
    assert_eq!(*avg_timing, Some(Duration::milliseconds(10500)));
}

#[test]
fn test_chunk_timing_stats_attempts_tracking() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    // Add samples with different attempt counts
    stats.add_timing(characteristics, Duration::seconds(5), 1);
    stats.add_timing(characteristics, Duration::seconds(5), 2);
    stats.add_timing(characteristics, Duration::seconds(5), 3);

    // Average attempts should be 2.0
    let all_stats = stats.get_statistics();
    let (_char, _avg_timing, avg_attempts) = &all_stats[0];
    assert_eq!(*avg_attempts, Some(2.0));
}

#[test]
fn test_chunk_timing_stats_clone() {
    let mut stats = ChunkTimingStats::new();

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: WaveformType::CS,
        channel_configuration: ChannelConfiguration::ConstantPhase,
    };

    stats.add_timing(characteristics, Duration::seconds(5), 1);

    let cloned = stats.clone();
    let cloned_stats = cloned.get_statistics();

    assert_eq!(cloned_stats.len(), 1);
    let (_char, avg_timing, _avg_attempts) = &cloned_stats[0];
    assert_eq!(*avg_timing, Some(Duration::seconds(5)));
}

#[test]
fn test_estimate_chunk_processing_time_start_chunk() {
    use chrono::NaiveDateTime;
    use nexrad_data::aws::realtime::{
        estimate_chunk_processing_time, ChunkIdentifier, VolumeIndex,
    };

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let start_chunk = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S").unwrap(),
        1,
        ChunkType::Start,
        None,
    );

    let estimate = estimate_chunk_processing_time(&start_chunk, &vcp, &mapper, None);

    // Start chunks should have a fixed estimate (10 seconds)
    assert!(estimate.is_some());
    assert_eq!(estimate.unwrap(), Duration::seconds(10));
}

#[test]
fn test_estimate_chunk_processing_time_with_stats() {
    use chrono::NaiveDateTime;
    use nexrad_data::aws::realtime::{
        estimate_chunk_processing_time, ChunkIdentifier, VolumeIndex,
    };

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    // Get characteristics for sequence 2
    let elevation = mapper.get_sequence_elevation_number(2).unwrap();
    let elev_data = &vcp.elevations()[elevation - 1];

    let characteristics = ChunkCharacteristics {
        chunk_type: ChunkType::Intermediate,
        waveform_type: elev_data.waveform_type(),
        channel_configuration: elev_data.channel_configuration(),
    };

    // Create stats with known timing
    let mut stats = ChunkTimingStats::new();
    stats.add_timing(characteristics, Duration::seconds(5), 2);

    let chunk = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S").unwrap(),
        2,
        ChunkType::Intermediate,
        None,
    );

    let estimate = estimate_chunk_processing_time(&chunk, &vcp, &mapper, Some(&stats));

    // Should use historical timing (5 seconds + 1 second for the extra attempt)
    assert!(estimate.is_some());
    assert_eq!(estimate.unwrap(), Duration::seconds(6)); // 5 + (2-1)
}

#[test]
fn test_estimate_chunk_processing_time_without_stats() {
    use chrono::NaiveDateTime;
    use nexrad_data::aws::realtime::{
        estimate_chunk_processing_time, ChunkIdentifier, VolumeIndex,
    };

    let vcp = get_test_vcp();
    let mapper = ElevationChunkMapper::new(&vcp);

    let chunk = ChunkIdentifier::new(
        "KDMX".to_string(),
        VolumeIndex::new(1),
        NaiveDateTime::parse_from_str("20220305_120000", "%Y%m%d_%H%M%S").unwrap(),
        2,
        ChunkType::Intermediate,
        None,
    );

    let estimate = estimate_chunk_processing_time(&chunk, &vcp, &mapper, None);

    // Should use default timing (4, 7, or 11 seconds depending on waveform/channel config)
    assert!(estimate.is_some());
    let duration = estimate.unwrap();

    // Should be one of the default values
    assert!(
        duration == Duration::seconds(4)
            || duration == Duration::seconds(7)
            || duration == Duration::seconds(11),
        "Expected default timing value, got: {} seconds",
        duration.num_seconds()
    );
}

// Integration tests that require network access and aws-polling feature
#[cfg(all(feature = "aws-polling", not(target_arch = "wasm32")))]
mod polling_tests {
    use futures::StreamExt;
    use nexrad_data::aws::realtime::{chunk_stream, PollConfig};
    use std::pin::pin;
    use std::time::Duration as StdDuration;

    #[tokio::test]
    #[ignore = "requires AWS access and takes time"]
    async fn test_chunk_stream_basic() {
        let config = PollConfig::new("KDMX");
        let stream = chunk_stream(config);
        let mut stream = pin!(stream);

        let mut chunk_count = 0;

        // Use timeout to limit test duration
        let result = tokio::time::timeout(StdDuration::from_secs(30), async {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(downloaded) => {
                        assert!(!downloaded.identifier.name().is_empty());
                        assert!(!downloaded.chunk.data().is_empty());
                        chunk_count += 1;

                        // Stop after receiving a few chunks
                        if chunk_count >= 3 {
                            break;
                        }
                    }
                    Err(e) => {
                        panic!("Stream error: {:?}", e);
                    }
                }
            }
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
        assert!(chunk_count > 0, "Expected to receive at least one chunk");
    }
}
