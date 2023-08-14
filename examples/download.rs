//! examples/download
//!
//! This example downloads a random chunk for some date/site and prints its size.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;

use nexrad::download::{download_chunk, list_chunks};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing chunks for {} on {}...", site, date);
    let metas = list_chunks(site, &date).await?;

    println!("Found {} chunks.", metas.len());
    if let Some(meta) = metas.first() {
        println!("Downloading {}...", meta.identifier());
        let compressed_chunk = download_chunk(meta).await?;

        println!("Chunk data size (bytes): {}", compressed_chunk.data().len());
        println!("Chunk data is compressed: {}", compressed_chunk.compressed());
    } else {
        println!("No chunks found for the specified date/site to download.");
    }

    Ok(())
}