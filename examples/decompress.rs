//! examples/decompress
//!
//! This example downloads a random file for some date/site, decompresses it, and prints its size.
//!

use chrono::NaiveDate;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::is_compressed;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Listing files for {} on {}...", site, date);
    let metas = list_files(site, &date).await?;

    println!("Found {} files.", metas.len());
    if let Some(meta) = metas.first() {
        println!("Downloading {}...", meta.identifier());
        let compressed_file = download_file(meta).await?;

        println!("Data file size (bytes): {}", compressed_file.len());
        println!(
            "Data file is compressed: {}",
            is_compressed(compressed_file.as_slice())
        );

        let decompressed_file = decompress_file(&compressed_file)?;
        println!(
            "Decompressed file data size (bytes): {}",
            decompressed_file.len()
        );
        println!(
            "Decompressed file data is compressed: {}",
            is_compressed(decompressed_file.as_slice())
        );
    } else {
        println!("No files found for the specified date/site to download.");
    }

    Ok(())
}
