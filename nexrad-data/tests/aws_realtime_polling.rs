#![cfg(all(feature = "aws", feature = "nexrad-decode", feature = "bzip2"))]

use chrono::Duration;
use nexrad_data::aws::realtime::{
    ChunkCharacteristics, ChunkTimingStats, ChunkType, ElevationChunkMapper,
};
use nexrad_decode::messages::volume_coverage_pattern::{ChannelConfiguration, WaveformType};

// Test data file
const TEST_NEXRAD_FILE: &[u8] = include_bytes!("KDMX20220305_232324_V06");

// Get a VCP from the test file
fn get_test_vcp() -> nexrad_decode::messages::volume_coverage_pattern::Message {
    let volume_file = nexrad_data::volume::File::new(TEST_NEXRAD_FILE.to_vec());

    for mut record in volume_file.records() {
        if record.compressed() {
            record = record.decompress().unwrap();
        }
        for message in record.messages().unwrap() {
            if let nexrad_decode::messages::MessageContents::VolumeCoveragePattern(vcp) =
                message.contents()
            {
                return *vcp.clone();
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
            elev_num >= 1 && elev_num <= vcp.elevations.len(),
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
        vcp.elevations.len(),
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
    let elev_data = &vcp.elevations[elevation - 1];

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

// Integration tests that require network access

#[tokio::test]
#[ignore = "requires AWS access and takes time"]
async fn test_poll_chunks_basic() {
    use nexrad_data::aws::realtime::{poll_chunks, Chunk, ChunkIdentifier};
    use std::sync::mpsc;
    use std::time::Duration as StdDuration;

    let site = "KDMX";
    let (tx, rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stats_tx, _stats_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    // Spawn polling in background
    let poll_handle =
        tokio::spawn(async move { poll_chunks(site, tx, Some(stats_tx), stop_rx).await });

    // Let it poll for a few seconds
    tokio::time::sleep(StdDuration::from_secs(30)).await;

    // Stop polling
    stop_tx.send(true).unwrap();

    // Wait for polling to finish
    let result = poll_handle.await.unwrap();
    assert!(result.is_ok(), "Polling failed: {:?}", result.err());

    // Should have received some chunks
    let chunk_count = rx.try_iter().count();
    assert!(chunk_count > 0, "Expected to receive at least one chunk");
}

#[test]
fn test_poll_stats_variants() {
    use nexrad_data::aws::realtime::{NewChunkStats, PollStats};

    // Test that all PollStats variants can be constructed
    let _latest_volume_calls = PollStats::LatestVolumeCalls(5);
    let _new_volume_calls = PollStats::NewVolumeCalls(3);
    let _new_chunk = PollStats::NewChunk(NewChunkStats {
        calls: 2,
        download_time: None,
        upload_time: None,
    });
    let _chunk_timings = PollStats::ChunkTimings(ChunkTimingStats::new());
}

#[test]
fn test_new_chunk_stats_latency() {
    use chrono::Utc;
    use nexrad_data::aws::realtime::NewChunkStats;

    let upload_time = Utc::now();
    let download_time = upload_time + Duration::seconds(5);

    let stats = NewChunkStats {
        calls: 1,
        download_time: Some(download_time),
        upload_time: Some(upload_time),
    };

    let latency = stats.latency();
    assert!(latency.is_some());

    // Note: latency is upload - download (negative for future downloads)
    // The actual calculation is upload_time.signed_duration_since(download_time)
    assert_eq!(latency.unwrap(), Duration::seconds(-5));
}

#[test]
fn test_new_chunk_stats_latency_missing_times() {
    use nexrad_data::aws::realtime::NewChunkStats;

    let stats = NewChunkStats {
        calls: 1,
        download_time: None,
        upload_time: None,
    };

    let latency = stats.latency();
    assert_eq!(latency, None);
}
