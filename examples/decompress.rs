//! examples/decompress
//!
//! This example downloads a random chunk for some date/site, decompresses it, and prints its size.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing chunks for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    println!("Found {} chunks.", metas.len());
    if let Some(meta) = metas.first() {
        println!("Downloading {}...", meta.identifier());
        let compressed_chunk = download_file(meta).await?;

        println!("Chunk data size (bytes): {}", compressed_chunk.len());
        println!(
            "Chunk data is compressed: {}",
            is_compressed(compressed_chunk.as_slice())
        );

        let decompressed_chunk = decompress_file(&compressed_chunk)?;
        println!(
            "Decompressed chunk data size (bytes): {}",
            decompressed_chunk.len()
        );
        println!(
            "Decompressed chunk data is compressed: {}",
            is_compressed(decompressed_chunk.as_slice())
        );
    } else {
        println!("No chunks found for the specified date/site to download.");
    }

    Ok(())
}
