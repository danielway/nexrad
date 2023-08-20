//! examples/decode
//!
//! This example downloads a random chunk and decodes it.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;

use nexrad::decode::decode_chunk;
use nexrad::download::{download_chunk, list_files};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing chunks for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    let meta = metas.first().expect("at least one chunk on date");
    println!("Found {} chunks. Downloading {}...", metas.len(), meta.identifier());

    let chunk = download_chunk(meta).await?;
    println!(
        "Downloaded {} chunk of size {} bytes.",
        if chunk.compressed() { "compressed " } else { "decompressed" },
        chunk.data().len()
    );

    let decoded = decode_chunk(&chunk)?;
    println!("Decoded chunk: {:?}", decoded);

    Ok(())
}