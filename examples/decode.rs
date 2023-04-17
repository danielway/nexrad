//! examples/decode
//!
//! This example gets a random chunk from cache and decodes it.
//!

use chrono::NaiveDate;

use nexrad::cache::{get_cache, list_cache};
use nexrad::decode::decode_chunk;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    let metas = list_cache("chunk_cache", site, &date)?;
    let meta = metas.first().expect("at least one chunk in cache");
    println!("Found {} chunks in cache. Using {}.", metas.len(), meta.identifier());

    let chunk = get_cache("chunk_cache", &meta)?;

    println!(
        "Loaded {} chunk of size {} bytes.",
        if chunk.compressed() { "compressed " } else { "decompressed" },
        chunk.data().len()
    );

    let decoded = decode_chunk(&chunk)?;
    println!("Decoded chunk: {:?}", decoded);

    Ok(())
}