use crate::aws::realtime::find_latest::find_latest;
use crate::aws::realtime::list_chunks_in_volume::list_chunks_in_volume;
use crate::aws::realtime::VolumeIndex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

/// Identifies the volume index with the most recent data for the specified radar site. Real-time
/// NEXRAD data is uploaded to a series of rotating volumes `1..=999`, each containing ~55 chunks.
/// This function performs a binary search to find the volume with the most recent upload time.
///
/// Returns `None` if none of the site's volumes contain any chunks.
pub async fn get_latest_volume(site: &str) -> crate::result::Result<Option<VolumeIndex>> {
    get_latest_volume_counting_calls(site)
        .await
        .map(|(volume, _)| volume)
}

/// Like [`get_latest_volume`], but also returns the number of network calls made to find the
/// latest volume so callers can account for them.
pub(crate) async fn get_latest_volume_counting_calls(
    site: &str,
) -> crate::result::Result<(Option<VolumeIndex>, usize)> {
    let calls = AtomicUsize::new(0);
    let latest_volume = find_latest(998, |volume| {
        calls.fetch_add(1, Relaxed);
        async move {
            let chunks = list_chunks_in_volume(site, VolumeIndex::new(volume + 1), 1).await?;
            Ok(chunks.first().and_then(|chunk| chunk.upload_date_time()))
        }
    })
    .await
    .map(|volume| volume.map(|index| VolumeIndex::new(index + 1)))?;

    Ok((latest_volume, calls.load(Relaxed)))
}
