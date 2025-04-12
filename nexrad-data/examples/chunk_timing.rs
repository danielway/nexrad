#![cfg(all(feature = "aws", feature = "decode"))]

use chrono::{DateTime, Utc};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, trace, LevelFilter};
use nexrad_data::aws::realtime::{
    download_chunk, get_latest_volume, list_chunks_in_volume, Chunk, VolumeIndex,
};
use nexrad_data::result::Result;
use nexrad_decode::summarize;
use std::{cmp::Ordering, collections::HashMap};

/// Example to analyze timing between chunks in a NEXRAD volume and inspect their contents.
/// Displays information about the time differences between consecutive chunks and decodes
/// the data within each chunk to show message summaries.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Site identifier (e.g., KDMX)
    #[arg(default_value = "KDMX")]
    site: String,

    /// Volume index to analyze. If not specified, will use volume before the latest.
    #[arg(long)]
    volume: Option<usize>,

    /// Maximum number of chunks to analyze (0 for all)
    #[arg(long, default_value = "10")]
    max_chunks: usize,

    /// Whether to show detailed message information
    #[arg(long, default_value = "false")]
    detailed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();
    let site = cli.site.clone();
    let max_chunks = cli.max_chunks;
    let detailed = cli.detailed;

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
        if let (Some(time_a), Some(time_b)) = (a.upload_date_time(), b.upload_date_time()) {
            time_a.cmp(&time_b)
        } else if a.upload_date_time().is_some() {
            Ordering::Less
        } else if b.upload_date_time().is_some() {
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

    // If max_chunks is set, limit the number of chunks to analyze
    let chunks_to_analyze = if max_chunks > 0 && max_chunks < chunks.len() {
        info!("Limiting analysis to first {} chunks", max_chunks);
        chunks.iter().take(max_chunks).cloned().collect::<Vec<_>>()
    } else {
        chunks
    };

    // Display chunk timing information and download/decode each chunk
    println!(
        "\n{:<20} {:<30} {:<15} {:<40}",
        "Chunk", "Modified Time (UTC)", "Time Since Previous", "Content Summary"
    );
    println!("{:-<110}", "");

    let mut prev_time: Option<DateTime<Utc>> = None;
    let mut vcps = std::collections::HashSet::new();
    let mut total_messages = 0;

    for chunk_id in chunks_to_analyze {
        let chunk_name = chunk_id.name();
        let modified_time = chunk_id.upload_date_time();

        // Calculate time difference
        let time_diff = if let Some(time) = modified_time {
            prev_time
                .map(|prev| {
                    let duration = time.signed_duration_since(prev);
                    format!("{:.2} seconds", duration.num_milliseconds() as f64 / 1000.0)
                })
                .unwrap_or_else(|| "N/A".to_string())
        } else {
            "N/A".to_string()
        };

        // Download and decode the chunk
        let download_time = Utc::now();
        let content_summary = match download_chunk(&site, &chunk_id).await {
            Ok((_, chunk)) => {
                let summary = decode_chunk(&chunk, download_time, detailed)?;
                // Collect VCP information
                if let Some(vcp) = summary.vcp {
                    vcps.insert(vcp);
                }
                total_messages += summary.message_count;
                summary.summary
            }
            Err(err) => format!("Failed to download: {}", err),
        };

        println!(
            "{:<20} {:<30} {:<15} {:<40}",
            chunk_name,
            modified_time
                .map(|time| time.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            time_diff,
            content_summary
        );

        // Store time for next iteration
        if let Some(time) = modified_time {
            prev_time = Some(time);
        }
    }

    // Display statistics about the analyzed chunks
    println!("\nAnalysis Summary:");
    println!("  Volume Coverage Patterns found: {:?}", vcps);
    println!("  Total messages decoded: {}", total_messages);

    Ok(())
}

/// Information extracted from a chunk
struct ChunkSummary {
    summary: String,
    message_count: usize,
    vcp: Option<String>,
}

/// Decodes a chunk and returns summary information
fn decode_chunk(
    chunk: &Chunk,
    download_time: DateTime<Utc>,
    detailed: bool,
) -> Result<ChunkSummary> {
    let mut message_count = 0;
    let mut vcp = None;
    let mut data_types = HashMap::new();
    let mut min_azimuth = f32::MAX;
    let mut max_azimuth = f32::MIN;
    let mut elevations = Vec::new();

    let summary = match chunk {
        Chunk::Start(file) => {
            debug!("Decoding volume start chunk");
            // Process records in the file
            for mut record in file.records() {
                if record.compressed() {
                    trace!("Decompressing LDM record...");
                    record = record.decompress()?;
                }

                let messages = record.messages()?;
                message_count += messages.len();

                let msg_summary = summarize::messages(messages.as_slice());

                // Extract VCP information
                if !msg_summary.volume_coverage_patterns.is_empty() {
                    // Use the debug format for VCPs
                    let vcp_str = msg_summary
                        .volume_coverage_patterns
                        .iter()
                        .map(|v| format!("{:?}", v))
                        .collect::<Vec<_>>()
                        .join(",");

                    vcp = Some(format!("VCP{}", vcp_str.replace("VCP", "")));
                }

                // Track azimuth range, elevation angles, and data types
                for group in &msg_summary.message_groups {
                    // Extract azimuth information
                    if let (Some(start_az), Some(end_az)) = (group.start_azimuth, group.end_azimuth)
                    {
                        min_azimuth = min_azimuth.min(start_az);
                        max_azimuth = max_azimuth.max(end_az);
                    }

                    // Extract elevation information
                    if let Some(elev) = group.elevation_angle {
                        // Round to 2 decimal places and add if not already present
                        let rounded_elev = (elev * 100.0).round() / 100.0;
                        if !elevations.contains(&rounded_elev) {
                            elevations.push(rounded_elev);
                        }
                    }

                    // Count data types
                    if let Some(dt) = &group.data_types {
                        for (key, count) in dt {
                            *data_types.entry(key.clone()).or_insert(0) += count;
                        }
                    }
                }

                // Print detailed message information if requested
                if detailed {
                    println!("\nChunk Contents:\n{}", msg_summary);

                    if let Some(earliest) = msg_summary.earliest_collection_time {
                        let latency = download_time.signed_duration_since(earliest);
                        println!(
                            "  Message latency: {:.2} seconds",
                            latency.num_milliseconds() as f64 / 1000.0
                        );
                    }
                }
            }

            // Format azimuth range
            let azimuth_info = if min_azimuth != f32::MAX && max_azimuth != f32::MIN {
                format!("Az: {:.1}°-{:.1}°", min_azimuth, max_azimuth)
            } else {
                "Az: N/A".to_string()
            };

            // Format elevation info
            let elev_info = if !elevations.is_empty() {
                if elevations.len() == 1 {
                    format!("El: {:.2}°", elevations[0])
                } else {
                    format!("El: {} angles", elevations.len())
                }
            } else {
                "El: N/A".to_string()
            };

            format!(
                "{} msgs, {} types, {}, {}",
                message_count,
                data_types.len(),
                azimuth_info,
                elev_info
            )
        }
        Chunk::IntermediateOrEnd(record) => {
            debug!("Decoding intermediate/end chunk");
            // Clone the record to avoid ownership issues
            let mut record_clone = record.clone();
            if record_clone.compressed() {
                trace!("Decompressing LDM record...");
                record_clone = record_clone.decompress()?;
            }

            let messages = record_clone.messages()?;
            message_count = messages.len();

            let msg_summary = summarize::messages(messages.as_slice());

            // Extract VCP information
            if !msg_summary.volume_coverage_patterns.is_empty() {
                // Use the debug format for VCPs
                let vcp_str = msg_summary
                    .volume_coverage_patterns
                    .iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join(",");

                vcp = Some(format!("VCP{}", vcp_str.replace("VCP", "")));
            }

            // Track azimuth range, elevation angles, and data types
            for group in &msg_summary.message_groups {
                // Extract azimuth information
                if let (Some(start_az), Some(end_az)) = (group.start_azimuth, group.end_azimuth) {
                    min_azimuth = min_azimuth.min(start_az);
                    max_azimuth = max_azimuth.max(end_az);
                }

                // Extract elevation information
                if let Some(elev) = group.elevation_angle {
                    // Round to 2 decimal places and add if not already present
                    let rounded_elev = (elev * 100.0).round() / 100.0;
                    if !elevations.contains(&rounded_elev) {
                        elevations.push(rounded_elev);
                    }
                }

                // Count data types
                if let Some(dt) = &group.data_types {
                    for (key, count) in dt {
                        *data_types.entry(key.clone()).or_insert(0) += count;
                    }
                }
            }

            // Print detailed message information if requested
            if detailed {
                println!("\nChunk Contents:\n{}", msg_summary);

                if let Some(earliest) = msg_summary.earliest_collection_time {
                    let latency = download_time.signed_duration_since(earliest);
                    println!(
                        "  Message latency: {:.2} seconds",
                        latency.num_milliseconds() as f64 / 1000.0
                    );
                }
            }

            // Format azimuth range
            let azimuth_info = if min_azimuth != f32::MAX && max_azimuth != f32::MIN {
                format!("Az: {:.1}°-{:.1}°", min_azimuth, max_azimuth)
            } else {
                "Az: N/A".to_string()
            };

            // Format elevation info
            let elev_info = if !elevations.is_empty() {
                if elevations.len() == 1 {
                    format!("El: {:.2}°", elevations[0])
                } else {
                    format!("El: {} angles", elevations.len())
                }
            } else {
                "El: N/A".to_string()
            };

            // List data type names (up to 3)
            let type_info = if !data_types.is_empty() {
                let mut type_names: Vec<_> = data_types.keys().cloned().collect();
                type_names.sort();
                let type_count = type_names.len();

                if type_count <= 3 {
                    format!("Types: {}", type_names.join(", "))
                } else {
                    format!(
                        "Types: {}, +{} more",
                        type_names
                            .iter()
                            .take(2)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", "),
                        type_count - 2
                    )
                }
            } else {
                "No data types".to_string()
            };

            format!(
                "{} msgs, {}, {}, {}",
                message_count, azimuth_info, elev_info, type_info
            )
        }
    };

    Ok(ChunkSummary {
        summary,
        message_count,
        vcp,
    })
}
