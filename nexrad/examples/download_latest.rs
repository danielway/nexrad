//! Download the latest radar scan from AWS.
//!
//! This example demonstrates using the facade API to download
//! NEXRAD data directly from the archive.
//!
//! Run with:
//! ```bash
//! cargo run --example download_latest -- KTLX
//! ```

use std::env;

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();

    let site = args.get(1).map(String::as_str).unwrap_or("KTLX");

    println!("Downloading latest scan for {}...", site);

    let scan = nexrad::download_latest(site).await?;

    println!("\n=== Downloaded Scan ===");
    println!("{}", scan.coverage_pattern_number());
    println!("Sweeps: {}", scan.sweeps().len());

    for (i, sweep) in scan.sweeps().iter().enumerate() {
        println!(
            "  Sweep {}: elevation {}, {} radials",
            i + 1,
            sweep.elevation_number(),
            sweep.radials().len()
        );
    }

    Ok(())
}
