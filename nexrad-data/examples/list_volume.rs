#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{list_chunks_in_volume, ChunkIdentifier, VolumeIndex};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    let chunks = list_chunks_in_volume("KDMX", VolumeIndex::new(961), 100).await?;
    println!("Found {} chunks", chunks.len());

    let mut previous_chunk: Option<ChunkIdentifier> = None;
    for chunk in chunks {
        if let Some(previous_chunk) = previous_chunk {
            let delta = chunk.date_time().unwrap() - previous_chunk.date_time().unwrap();
            println!("  {:?}: {:?}", chunk, delta);
        } else {
            println!("  {:?}", chunk);
        }
        previous_chunk = Some(chunk);
    }

    Ok(())
}
