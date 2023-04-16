//! examples/cache
//!
//! This example uses the caching utility to fetch all chunks for a date/site and save them to disk.
//!

use chrono::NaiveDate;

use nexrad::cache::{CacheConfig, update_cache};
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

    println!("Updating cached chunks for {} on {}...", site, date);
    let config = CacheConfig::new("chunk_cache")
        .fetch_handler(|meta| println!("Fetching {}...", meta.identifier()));
    let metas = update_cache(site, &date, config).await?;

    println!("Found and cached {} chunks.", metas.len());

    Ok(())
}