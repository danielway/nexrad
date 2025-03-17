use chrono::{DateTime, Utc};
use clap::Parser;
use log::{info, LevelFilter};
use nexrad_data::aws::realtime::{get_latest_volume, list_chunks_in_volume, VolumeIndex};
use std::cmp::Ordering;

/// Example to analyze timing between chunks in a NEXRAD volume.
/// Displays information about the time differences between consecutive chunks.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Site identifier (e.g., KDMX)
    #[arg(default_value = "KDMX")]
    site: String,

    /// Volume index to analyze. If not specified, will use volume before the latest.
    #[arg(long)]
    volume: Option<usize>,
}

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();
    let site = cli.site.clone();

    // Determine which volume to analyze
    let volume = if let Some(vol) = cli.volume {
        VolumeIndex::new(vol)
    } else {
        // Get the latest volume and use the previous one
        let latest_result = get_latest_volume(&site).await?;
        let latest = latest_result.volume.expect("No latest volume found");

        info!("Latest volume found: {}", latest.as_number());
        // Calculate previous volume (handle wrap around from 1 to 999)
        let prev_num = if latest.as_number() > 1 {
            latest.as_number() - 1
        } else {
            999
        };
        let prev = VolumeIndex::new(prev_num);
        info!("Using previous volume: {}", prev.as_number());

        prev
    };

    // List all chunks in the volume
    info!(
        "Listing chunks for site {} in volume {}",
        site,
        volume.as_number()
    );
    let mut chunks = list_chunks_in_volume(&site, volume, 1000).await?;

    // Sort chunks by modified time
    chunks.sort_by(|a, b| {
        if let (Some(time_a), Some(time_b)) = (a.date_time(), b.date_time()) {
            time_a.cmp(&time_b)
        } else if a.date_time().is_some() {
            Ordering::Less
        } else if b.date_time().is_some() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    info!(
        "Found {} chunks in volume {}",
        chunks.len(),
        volume.as_number()
    );

    if chunks.is_empty() {
        info!("No chunks found in this volume");
        return Ok(());
    }

    // Display chunk timing information
    println!(
        "\n{:<20} {:<30} {:<15}",
        "Chunk", "Modified Time (UTC)", "Time Since Previous"
    );
    println!("{:-<70}", "");

    let mut prev_time: Option<DateTime<Utc>> = None;

    for chunk in &chunks {
        let chunk_name = chunk.name();
        if let Some(time) = chunk.date_time() {
            let time_diff = prev_time
                .map(|prev| {
                    let duration = time.signed_duration_since(prev);
                    format!("{:.2} seconds", duration.num_milliseconds() as f64 / 1000.0)
                })
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "{:<20} {:<30} {:<15}",
                chunk_name,
                time.format("%Y-%m-%d %H:%M:%S%.3f"),
                time_diff
            );

            prev_time = Some(time);
        } else {
            println!("{:<20} {:<30} {:<15}", chunk_name, "Unknown", "N/A");
        }
    }

    // Calculate some basic statistics
    if chunks.len() > 1 {
        let mut intervals = Vec::new();
        let mut prev_chunk_time: Option<DateTime<Utc>> = None;

        for chunk in &chunks {
            if let Some(current_time) = chunk.date_time() {
                if let Some(prev_time) = prev_chunk_time {
                    let duration = current_time.signed_duration_since(prev_time);
                    intervals.push(duration.num_milliseconds() as f64 / 1000.0);
                }
                prev_chunk_time = Some(current_time);
            }
        }

        if !intervals.is_empty() {
            let sum: f64 = intervals.iter().sum();
            let avg = sum / intervals.len() as f64;

            let min = intervals.iter().fold(f64::MAX, |a, &b| a.min(b));
            let max = intervals.iter().fold(f64::MIN, |a, &b| a.max(b));

            println!("\nTiming Statistics:");
            println!("  Average interval: {:.2} seconds", avg);
            println!("  Minimum interval: {:.2} seconds", min);
            println!("  Maximum interval: {:.2} seconds", max);

            // Calculate first and last time
            if let (Some(first), Some(last)) = (
                chunks.first().and_then(|c| c.date_time()),
                chunks.last().and_then(|c| c.date_time()),
            ) {
                let total_duration = last.signed_duration_since(first);
                println!(
                    "  Total span: {:.2} minutes",
                    total_duration.num_seconds() as f64 / 60.0
                );
            }
        }
    }

    Ok(())
}
