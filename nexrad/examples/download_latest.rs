//! Download the latest radar volume from AWS.
//!
//! This example demonstrates using the AWS feature to download
//! NEXRAD data directly from the archive.
//!
//! Run with:
//! ```bash
//! cargo run --example download_latest --features nexrad/aws -- KTLX 2023-05-20
//! ```

use chrono::NaiveDate;
use nexrad::data::aws::archive;
use std::env;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();

    let site = args.get(1).map(String::as_str).unwrap_or("KTLX");
    let date_str = args.get(2).map(String::as_str).unwrap_or("2023-05-20");

    let date =
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Date format should be YYYY-MM-DD");

    println!("Fetching latest volume for {} on {}...", site, date);

    // List available volumes
    let volumes = nexrad::list_volumes(site, date).await?;
    println!("Found {} volumes for this date", volumes.len());

    if volumes.is_empty() {
        println!("No data available for {} on {}", site, date);
        return Ok(());
    }

    // Try latest files, falling back if incomplete
    let mut volume = None;
    for i in (0..volumes.len()).rev().take(3) {
        println!("Trying volume {}...", i + 1);
        let file = archive::download_file(volumes[i].clone()).await?;
        match file.scan() {
            Ok(v) => {
                volume = Some(v);
                break;
            }
            Err(e) => {
                println!("  Skipping ({})", e);
            }
        }
    }

    let volume = volume.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No complete volumes found")
    })?;

    println!("\n=== Downloaded Volume ===");
    println!("VCP: {}", volume.coverage_pattern_number());
    println!("Sweeps: {}", volume.sweeps().len());

    // Print first sweep info
    if let Some(sweep) = volume.sweeps().first() {
        println!(
            "First sweep: elevation {}, {} radials",
            sweep.elevation_number(),
            sweep.radials().len()
        );
    }

    Ok(())
}
