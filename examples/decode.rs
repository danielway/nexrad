//! examples/decode
//!
//! This example downloads a random data file and decodes it.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;

use nexrad::decode::decode_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing files for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    let meta = metas.first().expect("at least one file on date");
    println!(
        "Found {} files. Downloading {}...",
        metas.len(),
        meta.identifier()
    );

    let file = download_file(meta).await?;
    println!(
        "Downloaded {} file of size {} bytes.",
        if is_compressed(file.as_slice()) {
            "compressed"
        } else {
            "decompressed"
        },
        file.len()
    );

    let decoded = decode_file(&file)?;
    println!("Decoded file: {:?}", decoded);

    Ok(())
}
