use std::io::Cursor;
use nexrad_data::result::Result;
use nexrad_decode::messages::decode_message_header;

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{get_latest_volume, list_chunks, download_chunk};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> Result<()> {
    let latest = get_latest_volume("KDMX").await?;
    println!("Most recent volume: {:?}", latest);

    let chunks = list_chunks("KDMX", latest.unwrap(), 1000).await?;
    println!("Found {} chunks.", chunks.len());

    for chunk in &chunks {
        println!("{:?}", chunk);
    }

    let latest = chunks.last().unwrap();
    let file = download_chunk("KDMX", latest).await?;
    println!("Downloaded chunk size: {}", file.data().len());

    println!(
        "Writing chunk to downloads/{}...",
        latest.identifier().unwrap()
    );
    std::fs::write(
        format!("downloads/{}", latest.identifier().unwrap()),
        file.data(),
    )?;

    let mut cursor = Cursor::new(file.data());
    let message_header = decode_message_header(&mut cursor).unwrap();
    println!("Decoded message header: {:?}", message_header);
    
    Ok(())
}
