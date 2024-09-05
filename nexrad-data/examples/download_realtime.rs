use std::time::Duration;

use nexrad_data::aws::realtime::list_chunks_in_volume;
#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{download_chunk, get_latest_volume, get_next_chunk};

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
        next_chunk = get_next_chunk("KDMX", next_chunk.clone()).expect("No next chunk found");
        println!("Next chunk: {:?}", next_chunk);

        for attempt in 0..5 {
            println!("  Downloading chunk, attempt {}/5...", attempt + 1);
            match download_chunk("KDMX", &next_chunk).await {
                Ok(chunk) => {
                    println!("  Downloaded chunk: {} bytes", chunk.data().len());

                    downloaded_chunks += 1;
                    break;
                }
                Err(e) => {
                    println!("Error downloading chunk: {:?}", e);
                    if attempt == 4 {
                        return Err(e);
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        if downloaded_chunks >= 5 {
            break;
        }
    }

    println!("Done!");

    Ok(())
}
