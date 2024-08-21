use crate::aws::realtime::list_chunks_in_volume::list_chunks_in_volume;
use crate::aws::realtime::VolumeIndex;
use crate::aws::search::binary_search_greatest;

/// Identifies the volume index with the most recent data for the specified radar site. Real-time
/// NEXRAD data is uploaded to a series of rotating volumes 0..=999, each containing ~55 chunks.
/// This function performs a binary search to find the most recent volume with data.
pub async fn get_latest_volume(site: &str) -> crate::result::Result<Option<VolumeIndex>> {
    binary_search_greatest(1, 999, |volume| async move {
        let chunks = list_chunks_in_volume(site, VolumeIndex::new(volume), 1).await?;
        Ok(chunks.first().map(|chunk| chunk.date_time()))
    })
    .await
    .map(|volume| volume.map(VolumeIndex::new))
}
