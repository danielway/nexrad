use std::time::Duration;

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{download_chunk, get_latest_volume};
use nexrad_data::aws::realtime::{list_chunks_in_volume, ChunkIdentifier, NextChunk, VolumeIndex};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    println!("Querying for latest volume...");
    let latest_volume = get_latest_volume("KDMX").await?.expect("No volume found");
    println!("  Latest volume: {:?}", latest_volume);

    println!("Listing chunks in volume...");
    let chunks = list_chunks_in_volume("KDMX", latest_volume, 100).await?;
    println!("  Found {} chunks", chunks.len());
    let latest_chunk = chunks.last().expect("No chunks found");
    println!("  Latest chunk: {:?}", latest_chunk);

    let mut downloaded_chunks = 0;
    let mut next_chunk = latest_chunk.clone();
    loop {
        match next_chunk.next_chunk() {
            Some(next) => match next {
                NextChunk::Sequence(next_chunk_identifier) => {
                    println!("Next chunk: {:?}", next_chunk_identifier);
                    next_chunk = next_chunk_identifier;
                }
                NextChunk::Volume(next_volume) => {
                    println!("Next volume: {:?}", next_volume);
                    next_chunk = attempt_next_volume(next_volume).await?;
                }
            },
            None => {
                println!("Unable to determine next chunk.");
                break;
            }
        }

        attempt_chunk_download(&next_chunk).await?;

        if downloaded_chunks >= 5 {
            break;
        }
    }

    println!("Done!");

    Ok(())
}

#[cfg(feature = "aws")]
async fn attempt_next_volume(next_volume: VolumeIndex) -> nexrad_data::result::Result<ChunkIdentifier> {
    for attempt in 0..5 {
        println!(
            "  Looking for chunks in volume {:?}, attempt {}/5...",
            next_volume,
            attempt + 1
        );
        match list_chunks_in_volume("KDMX", next_volume, 100).await {
            Ok(chunks) => match chunks.last() {
                Some(chunk) => {
                    println!("    Found latest chunk: {:?}", chunk);
                    return Ok(chunk.clone());
                }
                None => {
                    println!("    No chunks found in volume.");
                    continue;
                }
            },
            Err(e) => {
                println!("    Error downloading chunk: {:?}", e);
                if attempt == 4 {
                    return Err(e);
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    panic!("Unable to find chunk in volume");
}

#[cfg(feature = "aws")]
async fn attempt_chunk_download(next_chunk: &ChunkIdentifier) -> nexrad_data::result::Result<()> {
    for attempt in 0..5 {
        println!(
            "  Downloading chunk {}, attempt {}/5...",
            next_chunk.name(),
            attempt + 1
        );
        match download_chunk("KDMX", &next_chunk).await {
            Ok(chunk) => {
                println!("    Downloaded chunk: {} bytes", chunk.data().len());
                return Ok(());
            }
            Err(e) => {
                println!("    Error downloading chunk: {:?}", e);
                if attempt == 4 {
                    return Err(e);
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    panic!("Unable to download chunk");
}
