//! examples/decode
//!
//! This example downloads a random chunk and decodes it.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;

use nexrad::decode::decode_chunk;
use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing chunks for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    let meta = metas.first().expect("at least one chunk on date");
    println!(
        "Found {} chunks. Downloading {}...",
        metas.len(),
        meta.identifier()
    );

    let chunk = download_file(meta).await?;
    println!(
        "Downloaded {} chunk of size {} bytes.",
        if is_compressed(chunk.as_slice()) {
            "compressed"
        } else {
            "decompressed"
        },
        chunk.len()
    );

    let decoded = decode_chunk(&chunk)?;
    println!("Decoded chunk: {:?}", decoded);

    Ok(())
}
