use chrono::Utc;
use std::sync::mpsc;
use std::time::Duration;
use tokio::task;

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{poll_chunks, Chunk, ChunkIdentifier};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    let mut downloaded_chunks = 0;

    let (update_tx, update_rx) = mpsc::channel::<(ChunkIdentifier, Chunk)>();
    let (stop_tx, stop_rx) = mpsc::channel::<bool>();

    task::spawn(async move {
        poll_chunks("KDMX", update_tx, stop_rx)
            .await
            .expect("Failed to poll chunks");
    });

    let timeout_stop_tx = stop_tx.clone();
    task::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;

        println!("Timeout reached, stopping...");
        timeout_stop_tx.send(true).unwrap();
    });

    let handle = task::spawn(async move {
        loop {
            let (chunk_id, chunk) = update_rx.recv().expect("Failed to receive update");

            println!(
                "Downloaded chunk {} from {:?} at {:?} of size {}",
                chunk_id.name(),
                chunk_id.date_time(),
                Utc::now(),
                chunk.data().len()
            );

            downloaded_chunks += 1;
            if downloaded_chunks >= 10 {
                println!("Downloaded 10 chunks, stopping...");
                stop_tx.send(true).expect("Failed to send stop signal");
                break;
            }
        }
    });

    handle.await.expect("Failed to join handle");

    println!("Finished downloading chunks");

    Ok(())
}
