//! examples/decode
//!
//! This example fetches a random chunk and decodes it.
//!

use chrono::NaiveDate;

use nexrad::decode::decode_chunk;
use nexrad::fetch::{fetch_chunk, list_chunks};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing chunks for {} on {}...", site, date);
    let metas = list_chunks(site, &date).await?;

    let meta = metas.first().expect("at least one chunk on date");
    println!("Found {} chunks. Downloading {}...", metas.len(), meta.identifier());

    let chunk = fetch_chunk(meta).await?;
    println!(
        "Downloaded {} chunk of size {} bytes.",
        if chunk.compressed() { "compressed " } else { "decompressed" },
        chunk.data().len()
    );

    let decoded = decode_chunk(&chunk)?;
    println!("Decoded chunk: {:?}", decoded);

    Ok(())
}