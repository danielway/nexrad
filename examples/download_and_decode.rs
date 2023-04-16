use chrono::NaiveDate;

use nexrad::chunk::{Chunk, EncodedChunk};
use nexrad::decode::decode_chunk;
use nexrad::fetch::{fetch_chunk, list_chunks};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Downloading chunks for {} on {}...", "KDMX", date);
    let encoded_chunks = download_chunks("KDMX", &date).await?;

    println!("Downloaded {} chunks. Decoding...", encoded_chunks.len());
    let decoded_chunks = decode_chunks(encoded_chunks)?;
    println!("Decoded {} chunks.", decoded_chunks.len());

    Ok(())
}

async fn download_chunks(site: &str, date: &NaiveDate) -> Result<Vec<EncodedChunk>> {
    let metas = list_chunks(site, date).await?;
    println!("Found {} chunks. Beginning downloads...", metas.len());

    let mut chunks = Vec::new();
    for meta in metas {
        println!("Downloading chunk {:?}...", meta);
        let chunk = fetch_chunk(&meta).await?;
        chunks.push(chunk);
    }

    Ok(chunks)
}

fn decode_chunks(encoded_chunks: Vec<EncodedChunk>) -> Result<Vec<Chunk>> {
    encoded_chunks.iter().map(|encoded_chunk| decode_chunk(encoded_chunk)).collect()
}