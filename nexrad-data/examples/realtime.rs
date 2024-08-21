#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{download_chunk, get_latest_volume, list_chunks_in_volume, Chunk};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    use nexrad_decode::messages::decode_message_header;
    use std::io::Cursor;

    let latest = get_latest_volume("KDMX").await?;
    println!("Most recent volume: {:?}", latest);

    let chunks = list_chunks("KDMX", latest.unwrap(), 1000).await?;
    println!("Found {} chunks.", chunks.len());

    for chunk in &chunks {
        println!("  {:?}", chunk);
    }

    let latest = chunks.last().unwrap();
    println!("Downloading chunk: {:?}", latest);

    let chunk = download_chunk("KDMX", latest).await?;
    println!("  Downloaded chunk size: {}", chunk.data().len());

    println!("  Writing chunk to downloads/{}...", latest.name());
    std::fs::write(format!("downloads/{}", latest.name()), &chunk.data())?;

    match chunk {
        Chunk::Start(file) => {
            println!("Start chunk volume header: {:?}", file.header());

            let records = file.records();
            let first_record = records.first().unwrap();
            let decompressed_record = first_record.decompress()?;

            let message_header =
                decode_message_header(&mut Cursor::new(decompressed_record.data())).unwrap();
            println!("Decoded message header: {:?}", message_header);
        }
        Chunk::IntermediateOrEnd(record) => {
            println!("Record is compressed: {}", record.compressed());
            let decompressed_record = record.decompress()?;

            let message_header =
                decode_message_header(&mut Cursor::new(decompressed_record.data())).unwrap();
            println!("Decoded message header: {:?}", message_header);
        }
    }

    Ok(())
}
