use chrono::NaiveDate;
use nexrad::chunk::{DecodedChunkFile, EncodedChunkFile};
use nexrad::decode::decode_chunk;
use nexrad::fetch::{fetch_chunk, list_chunks};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Downloading chunks for {} on {}...", "KDMX", date);
    let encoded_chunks = download_chunks("KDMX", date).await?;

    println!("Downloaded {} chunks. Decoding...", encoded_chunks.len());
    let decoded_chunks = decode_chunks(encoded_chunks)?;
    println!("Decoded {} chunks.", decoded_chunks.len());

    Ok(())
}

async fn download_chunks(site: &str, date: NaiveDate) -> Result<Vec<EncodedChunkFile>> {
    let metadatas = list_chunks(site, date).await?;
    println!("Found {} chunks. Beginning downloads...", metadatas.len());

    let mut chunks = Vec::new();
    for metadata in metadatas {
        println!("Downloading chunk {:?}...", metadata);
        let chunk = fetch_chunk(&metadata).await?;
        chunks.push(chunk);
    }

    Ok(chunks)
}

fn decode_chunks(encoded_chunks: Vec<EncodedChunkFile>) -> Result<Vec<DecodedChunkFile>> {
    encoded_chunks.iter().map(|encoded_chunk| decode_chunk(encoded_chunk)).collect()
}