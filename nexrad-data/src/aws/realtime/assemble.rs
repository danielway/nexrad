//! Assembles real-time data chunks into a complete volume scan.

use crate::aws::realtime::Chunk;
use crate::volume::{add_record_to_scan, build_scan};
use nexrad_decode::model::ScanBuilder;

/// Assembles a collection of real-time data chunks into a [`Scan`](nexrad_model::data::Scan).
///
/// Takes an iterator of chunks (typically collected from a [`ChunkIterator`](super::ChunkIterator)
/// or `chunk_stream`) and combines them into a complete volume scan.
///
/// The chunks should all belong to the same volume and should include a start chunk.
/// Chunks are processed in the order provided; radials from all chunks are combined
/// and grouped into sweeps by elevation.
///
/// # Errors
///
/// Returns an error if:
/// - No start chunk is found, so no volume coverage pattern was seen
///   ([`Error::MissingCoveragePattern`](crate::result::Error::MissingCoveragePattern))
/// - Decompression fails for any chunk, or any message in a chunk cannot be decoded
///   ([`Error::Decode`](crate::result::Error::Decode), carrying the first decoding error)
///
/// # Example
///
/// ```no_run
/// use nexrad_data::aws::realtime::{assemble_volume, ChunkIterator};
///
/// # async fn run() -> nexrad_data::Result<()> {
/// let init = ChunkIterator::start("KTLX").await?;
/// // The Start chunk carries the volume coverage pattern, so it comes first
/// let mut chunks: Vec<_> = init.start_chunk.into_iter().chain([init.latest_chunk]).collect();
/// // ... collect the rest of the volume with `init.iterator.try_next()` ...
/// let scan = assemble_volume(chunks.into_iter().map(|dc| dc.chunk))?;
/// # Ok(())
/// # }
/// ```
pub fn assemble_volume(
    chunks: impl IntoIterator<Item = Chunk>,
) -> crate::result::Result<nexrad_model::data::Scan> {
    let mut builder = ScanBuilder::new();
    for chunk in chunks {
        match chunk {
            Chunk::Start(file) => {
                for record in file.records()? {
                    add_record_to_scan(&mut builder, &record)?;
                }
            }
            Chunk::IntermediateOrEnd(record) => add_record_to_scan(&mut builder, &record)?,
        }
    }
    build_scan(builder)
}
