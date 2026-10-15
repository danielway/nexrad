use crate::aws::realtime::find_latest::find_latest;
use crate::aws::realtime::list_chunks_in_volume::list_chunks_in_volume;
use crate::aws::realtime::VolumeIndex;
use chrono::{DateTime, Utc};
use std::future::Future;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

/// The number of volume directories in the real-time bucket.
const VOLUME_COUNT: usize = 999;

/// Represents the most recent volume index and the number of network calls made to find it.
///
/// The fields are public so they can be read and moved out. The struct is `#[non_exhaustive]` so
/// fields may be added in a minor release.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct LatestVolumeResult {
    /// The most recent volume index, if found.
    pub volume: Option<VolumeIndex>,

    /// The number of network (list) calls made to find the most recent volume.
    pub calls: usize,
}

/// Identifies the volume index with the most recent data for the specified radar site. Real-time
/// NEXRAD data is uploaded to a series of rotating volumes `1..=999`, each containing ~55 chunks.
/// This function performs a binary search to find the volume with the most recent upload time.
///
/// The result's `volume` is `None` if none of the site's volumes contain any chunks.
///
/// # Errors
///
/// Returns an [`AwsError`](crate::result::AwsError) if a volume listing request fails or its
/// response cannot be parsed.
pub async fn get_latest_volume(site: &str) -> crate::result::Result<LatestVolumeResult> {
    let calls = AtomicUsize::new(0);
    let latest_volume = find_latest_volume(|volume| {
        calls.fetch_add(1, Relaxed);
        async move {
            let chunks = list_chunks_in_volume(site, volume).await?;
            Ok(chunks.first().and_then(|chunk| chunk.upload_date_time()))
        }
    })
    .await?;

    Ok(LatestVolumeResult {
        volume: latest_volume,
        calls: calls.load(Relaxed),
    })
}

/// Searches every volume `1..=999` for the one whose `probe` value is greatest, where `probe`
/// resolves to a volume's upload time or `None` if the volume is empty.
async fn find_latest_volume<F>(
    mut probe: impl FnMut(VolumeIndex) -> F,
) -> crate::result::Result<Option<VolumeIndex>>
where
    F: Future<Output = crate::result::Result<Option<DateTime<Utc>>>>,
{
    // Element `i` of the searched sequence is volume `i + 1`
    find_latest(VOLUME_COUNT, |element| {
        let probed = VolumeIndex::new(element + 1).map(&mut probe);
        async move { probed?.await }
    })
    .await?
    .map(|element| VolumeIndex::new(element + 1))
    .transpose()
}

/// Identifies the volume index with the most recent data, starting from a previously known
/// volume.
///
/// This is the fast path for a client that persisted the last volume it saw. Starting at `hint`,
/// it walks forward one volume at a time while the next volume holds a newer upload than the
/// current one, and returns the last volume reached. When the hint is minutes old this typically
/// takes 1 to 3 list calls, compared with the logarithmic search of [`get_latest_volume`].
///
/// If the hint volume has no chunks (or no upload time), this falls back to
/// [`get_latest_volume`] and returns its result, with `calls` including the probe of the hint.
///
/// This function does not judge whether the hint is too old to be useful. A very stale hint still
/// works but costs one list call per volume walked; deciding when to skip the hint and call
/// [`get_latest_volume`] directly is the caller's responsibility.
///
/// # Errors
///
/// Returns an [`AwsError`](crate::result::AwsError) if a volume listing request fails or its
/// response cannot be parsed.
pub async fn get_latest_volume_from_hint(
    site: &str,
    hint: VolumeIndex,
) -> crate::result::Result<LatestVolumeResult> {
    let mut calls = 1;
    let Some(mut candidate_time) = newest_upload_time(site, hint).await? else {
        let full = get_latest_volume(site).await?;
        return Ok(LatestVolumeResult {
            volume: full.volume,
            calls: full.calls + calls,
        });
    };

    let mut candidate = hint;
    // Bound the walk to one lap of the bucket so it terminates even if every volume appears newer.
    for _ in 1..VOLUME_COUNT {
        let next = candidate.next();
        let next_time = newest_upload_time(site, next).await?;
        calls += 1;

        if !should_advance(candidate_time, next_time) {
            break;
        }

        candidate = next;
        if let Some(time) = next_time {
            candidate_time = time;
        }
    }

    Ok(LatestVolumeResult {
        volume: Some(candidate),
        calls,
    })
}

/// Returns the upload time of the newest chunk in `volume`, or `None` if it has no chunks.
async fn newest_upload_time(
    site: &str,
    volume: VolumeIndex,
) -> crate::result::Result<Option<DateTime<Utc>>> {
    let chunks = list_chunks_in_volume(site, volume).await?;
    Ok(chunks.last().and_then(|chunk| chunk.upload_date_time()))
}

/// Whether the hint walk should advance from a volume whose newest upload is `current` to the
/// next volume, whose newest upload is `next` (`None` if it is empty).
fn should_advance(current: DateTime<Utc>, next: Option<DateTime<Utc>>) -> bool {
    next.is_some_and(|next| next > current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, second)
            .single()
            .unwrap()
    }

    #[test]
    fn advances_only_to_strictly_newer_volumes() {
        assert!(should_advance(at(10), Some(at(11))));
        assert!(!should_advance(at(10), Some(at(10))));
        assert!(!should_advance(at(10), Some(at(9))));
    }

    #[test]
    fn stops_at_empty_volume() {
        assert!(!should_advance(at(10), None));
    }

    /// Runs the volume search over in-memory upload times, where `times[n - 1]` is volume `n`.
    async fn latest_of(times: &[Option<DateTime<Utc>>]) -> Option<usize> {
        find_latest_volume(|volume| {
            let time = times[volume.as_number() - 1];
            async move { Ok(time) }
        })
        .await
        .unwrap()
        .map(|volume| volume.as_number())
    }

    /// Upload times for a full bucket rotation whose newest volume is `newest`.
    fn rotation_ending_at(newest: usize) -> Vec<Option<DateTime<Utc>>> {
        (1..=VOLUME_COUNT)
            .map(|volume| {
                let age = (newest + VOLUME_COUNT - volume) % VOLUME_COUNT;
                Some(at(0) - chrono::Duration::minutes(age as i64))
            })
            .collect()
    }

    #[tokio::test]
    async fn search_covers_every_volume() {
        for newest in [1, 2, 500, 998, 999] {
            assert_eq!(
                latest_of(&rotation_ending_at(newest)).await,
                Some(newest),
                "newest volume {newest}"
            );
        }
    }

    #[tokio::test]
    async fn search_finds_volume_999_before_a_gap() {
        let mut times = rotation_ending_at(999);
        for time in &mut times[..100] {
            *time = None;
        }
        assert_eq!(latest_of(&times).await, Some(999));
    }

    #[tokio::test]
    async fn search_of_empty_bucket_finds_nothing() {
        assert_eq!(latest_of(&vec![None; VOLUME_COUNT]).await, None);
    }
}
