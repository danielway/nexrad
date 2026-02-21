//! Download the latest radar volume from AWS.
//!
//! This example demonstrates using the facade API to download
//! NEXRAD data directly from the archive.
//!
//! Run with:
//! ```bash
//! cargo run --example download_latest --features nexrad/aws -- KTLX 2023-05-20
//! ```

use chrono::NaiveDate;
use std::env;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();

    let site = args.get(1).map(String::as_str).unwrap_or("KTLX");
    let date_str = args.get(2).map(String::as_str).unwrap_or("2023-05-20");

    let date =
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").expect("Date format should be YYYY-MM-DD");

    println!("Downloading latest volume for {} on {}...", site, date);

    let volume = nexrad::download_latest(site, date).await?;

    println!("\n=== Downloaded Volume ===");
    println!("VCP: {}", volume.coverage_pattern_number());
    println!("Sweeps: {}", volume.sweeps().len());

    for (i, sweep) in volume.sweeps().iter().enumerate() {
        println!(
            "  Sweep {}: elevation {}, {} radials",
            i + 1,
            sweep.elevation_number(),
            sweep.radials().len()
        );
    }

    Ok(())
}
