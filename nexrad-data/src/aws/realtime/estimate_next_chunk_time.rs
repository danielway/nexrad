use chrono::{DateTime, Utc};
use crate::aws::realtime::ChunkIdentifier;

/// Attempts to estimate the time at which the next chunk will be available given 2 or more
/// preceding chunks. 
pub fn estimate_next_chunk_time(chunks: Vec<&ChunkIdentifier>) -> Option<DateTime<Utc>> {
    if chunks.len() < 2 {
        return None;
    }

    let mut times: Vec<DateTime<Utc>> = chunks.iter().filter_map(|chunk| chunk.date_time()).collect();
    times.sort();

    let mut millisecond_spacings: Vec<i64> = Vec::new();
    for i in 1..times.len() {
        let timestamp = times[i].timestamp_millis();
        let previous_timestamp = times[i - 1].timestamp_millis();
        millisecond_spacings.push(timestamp - previous_timestamp);
    }

    let sum = millisecond_spacings.iter().sum::<i64>();
    let average = sum / millisecond_spacings.len() as i64;
    
    let last_time = times[times.len() - 1];
    Some(last_time + chrono::Duration::milliseconds(average))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;
    use crate::aws::realtime::VolumeIndex;
    
    macro_rules! chunk_identifier {
        ($date_time:expr) => {
            ChunkIdentifier::new(
                "KDMX".to_string(),
                VolumeIndex::new(1),
                "".to_string(),
                Some($date_time),
            )
        };
    }

    #[test]
    fn test_estimate_next_chunk_time() {
        let chunk1 = chunk_identifier!(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
        let chunk2 = chunk_identifier!(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 2).unwrap());
        let chunk3 = chunk_identifier!(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 4).unwrap());

        let result = estimate_next_chunk_time(vec![&chunk1, &chunk2, &chunk3]);
        assert_eq!(result, Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 6).unwrap()));
    }
}
