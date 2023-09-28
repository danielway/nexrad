//! examples/download
//!
//! This example downloads a data file for some date/site and prints its size.
//!

#![cfg(all(feature = "download"))]

use chrono::NaiveDate;
use std::io::Write;

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
        println!("Downloading the first file: {}...", meta.identifier());
        let downloaded_file = download_file(meta).await?;

        println!("Data file size (bytes): {}", downloaded_file.len());

        let is_compressed = is_compressed(downloaded_file.as_slice());
        println!("File data is compressed: {}", is_compressed);

        println!("Writing file to disk as: {}", meta.identifier());
        let mut file = std::fs::File::create(meta.identifier()).expect("create file");
        file.write_all(downloaded_file.as_slice())
            .expect("write file");
    } else {
        println!("No files found for the specified date/site to download.");
    }

    Ok(())
}
