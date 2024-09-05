use chrono::Utc;
use std::time::Duration;
use tokio::time::{sleep, sleep_until, Instant};

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{
    estimate_next_chunk_time, get_latest_volume, list_chunks_in_volume, ChunkIdentifier, NextChunk,
    VolumeIndex,
};

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
        let time_until_next_chunk = estimate_next_chunk_time(&next_chunk)
            .map(|time| time.signed_duration_since(Utc::now()).to_std().ok())
            .flatten();

        match time_until_next_chunk {
            Some(time_until) => {
                println!(
                    "Next chunk estimated to be available in {:?} at {:?}; sleeping",
                    time_until,
                    Utc::now() + time_until
                );
                sleep_until(Instant::now() + time_until).await;
            }
            None => {
                println!("Unable to estimate next chunk time; sleeping two seconds");
                sleep(Duration::from_secs(2)).await;
            }
        }

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

        next_chunk = attempt_get_chunk(&next_chunk).await?;

        downloaded_chunks += 1;
        if downloaded_chunks >= 100 {
            break;
        }
    }

    println!("Done!");

    Ok(())
}

#[cfg(feature = "aws")]
async fn attempt_next_volume(
    next_volume: VolumeIndex,
) -> nexrad_data::result::Result<ChunkIdentifier> {
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
                }
            },
            Err(e) => {
                println!("    Error downloading chunk: {:?}", e);
                if attempt == 4 {
                    return Err(e);
                }
            }
        }

        let wait = 500 * 2u64.pow(attempt);
        println!("    No chunks found in volume yet, retrying in {}ms", wait);
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }

    panic!("Unable to find chunk in volume");
}

#[cfg(feature = "aws")]
async fn attempt_get_chunk(
    next_chunk: &ChunkIdentifier,
) -> nexrad_data::result::Result<ChunkIdentifier> {
    for attempt in 0..5 {
        println!(
            "  Getting chunk {}, attempt {}/5...",
            next_chunk.name(),
            attempt + 1
        );

        let chunks = list_chunks_in_volume("KDMX", next_chunk.volume().clone(), 100).await?;
        let matching_chunks = chunks
            .into_iter()
            .filter(|chunk| chunk.name() == next_chunk.name())
            .collect::<Vec<_>>();
        match matching_chunks.first() {
            Some(chunk) => {
                let latency = Utc::now().signed_duration_since(chunk.date_time().unwrap());
                println!("    Found chunk: {:?} Latency: {:?}", chunk, latency);
                return Ok(chunk.clone());
            }
            None => {}
        }

        let wait = 500 * 2u64.pow(attempt);
        println!(
            "    No matching chunk found in volume yet, retrying in {}ms",
            wait
        );
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }

    panic!("Unable to find chunk");
}
