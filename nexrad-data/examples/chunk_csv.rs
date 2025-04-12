#![cfg(all(feature = "aws", feature = "decode"))]

use chrono::{DateTime, Utc};
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, warn, LevelFilter};
use nexrad_data::aws::realtime::{
    download_chunk, get_latest_volume, list_chunks_in_volume, Chunk, ChunkIdentifier,
    ElevationChunkMapper, VolumeIndex,
};
use nexrad_data::result::Result;
use nexrad_decode::messages::{volume_coverage_pattern, MessageContents};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// A tool to generate a CSV dataset of NEXRAD chunks with timing information and metadata.
/// This script analyzes chunk timing, contents, and maps them to VCP elevation cuts.
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
    #[arg(long, default_value = "0")]
    max_chunks: usize,

    /// Output CSV file path
    #[arg(long, default_value = "nexrad_chunks.csv")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("reqwest::connect", LevelFilter::Info)
        .init();

    let cli = Cli::parse();
    let site = cli.site.clone();
    let max_chunks = cli.max_chunks;
    let output_path = cli.output.clone();

    let volume = if let Some(vol) = cli.volume {
        VolumeIndex::new(vol)
    } else {
        let latest_result = get_latest_volume(&site).await?;
        let latest = latest_result.volume.expect("No latest volume found");
        info!("Latest volume found: {}", latest.as_number());

        let prev_num = if latest.as_number() > 1 {
            latest.as_number() - 1
        } else {
            999
        };

        let prev = VolumeIndex::new(prev_num);
        info!("Using previous volume: {}", prev.as_number());

        prev
    };

    info!(
        "Listing chunks for site {} in volume {}",
        site,
        volume.as_number()
    );
    let mut chunks = list_chunks_in_volume(&site, volume, 1000).await?;

    info!(
        "Found {} chunks in volume {}",
        chunks.len(),
        volume.as_number()
    );

    if chunks.is_empty() {
        info!("No chunks found in this volume");
        return Ok(());
    }

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

    let chunks_to_analyze = if max_chunks > 0 && max_chunks < chunks.len() {
        info!("Limiting analysis to first {} chunks", max_chunks);
        chunks.iter().take(max_chunks).cloned().collect::<Vec<_>>()
    } else {
        chunks
    };

    let mut file = File::create(&output_path)?;

    writeln!(
        file,
        "chunk_name,modified_time,time_since_previous_s,message_types,data_types,\
        earliest_message_time,latest_message_time,scan_time_s,processing_time_s,\
        elevation_numbers,matched_to_vcp,elevation_angle,azimuth_range,\
        vcp_number,channel_configuration,waveform_type,super_resolution,azimuth_rate"
    )?;

    info!(
        "Analyzing {} chunks and writing CSV to {}",
        chunks_to_analyze.len(),
        output_path.display()
    );

    let mut previous_time: Option<DateTime<Utc>> = None;
    let mut vcp: Option<volume_coverage_pattern::Message> = None;
    let mut elevation_chunk_mapper: Option<ElevationChunkMapper> = None;

    for (i, chunk_id) in chunks_to_analyze.iter().enumerate() {
        let chunk_name = chunk_id.name();
        let modified_time = chunk_id.date_time();

        // Calculate time difference from previous chunk
        let time_diff = if let Some(time) = modified_time {
            previous_time
                .map(|prev| {
                    let duration = time.signed_duration_since(prev);
                    duration.num_milliseconds() as f64 / 1000.0
                })
                .unwrap_or(0.0)
                .to_string()
        } else {
            "Unknown".to_string()
        };

        info!(
            "Processing chunk {}/{}: {}",
            i + 1,
            chunks_to_analyze.len(),
            chunk_name
        );

        match download_chunk(&site, chunk_id).await {
            Ok((_, chunk)) => {
                let result =
                    analyze_chunk(&chunk, chunk_id, &mut vcp, &mut elevation_chunk_mapper)?;

                write_csv_row(
                    &mut file,
                    chunk_name.to_string(),
                    modified_time,
                    time_diff,
                    result,
                )?;

                previous_time = modified_time;
            }
            Err(err) => {
                warn!("Failed to download chunk {}: {}", chunk_name, err);
                writeln!(
                    file,
                    "{},{},{},,,,,,,,,,,,",
                    chunk_name,
                    modified_time
                        .map(|time| time.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    time_diff
                )?;
            }
        }
    }

    info!("CSV data written to {}", output_path.display());
    Ok(())
}

/// Holds the analysis results for a chunk
struct ChunkAnalysis {
    message_types: Vec<String>,
    data_types: Vec<String>,
    earliest_message_time: Option<DateTime<Utc>>,
    latest_message_time: Option<DateTime<Utc>>,
    scan_time: Option<f64>,
    processing_time: Option<f64>,
    elevation_numbers: HashSet<u8>,
    elevation_angle: Option<f64>,
    matched_to_vcp: bool,
    azimuth_range: Option<f64>,
    vcp_number: Option<String>,
    channel_configuration: Option<String>,
    waveform_type: Option<String>,
    super_resolution: Option<String>,
    azimuth_rate: Option<f64>,
}

/// Analyzes a chunk and returns structured data about its contents
fn analyze_chunk(
    chunk: &Chunk,
    chunk_id: &ChunkIdentifier,
    vcp: &mut Option<volume_coverage_pattern::Message>,
    elevation_chunk_mapper: &mut Option<ElevationChunkMapper>,
) -> Result<ChunkAnalysis> {
    let mut result = ChunkAnalysis {
        message_types: Vec::new(),
        data_types: Vec::new(),
        earliest_message_time: None,
        latest_message_time: None,
        scan_time: None,
        processing_time: None,
        elevation_numbers: HashSet::new(),
        elevation_angle: None,
        matched_to_vcp: false,
        azimuth_range: None,
        vcp_number: None,
        channel_configuration: None,
        waveform_type: None,
        super_resolution: None,
        azimuth_rate: None,
    };

    let mut messages = Vec::new();
    match chunk {
        Chunk::Start(file) => {
            for mut record in file.records() {
                if record.compressed() {
                    record = record.decompress()?;
                }

                messages.extend(record.messages()?);
            }
        }
        Chunk::IntermediateOrEnd(record) => {
            let mut record = record.clone();
            if record.compressed() {
                record = record.decompress()?;
            }

            messages.extend(record.messages()?);
        }
    }

    let mut message_type_counter = HashMap::new();
    let mut data_type_counter = HashMap::new();
    let mut radar_times = Vec::new();

    for message in &messages {
        let msg_type = message.header().message_type();
        *message_type_counter.entry(msg_type).or_insert(0) += 1;

        match message.contents() {
            MessageContents::VolumeCoveragePattern(chunk_vcp) => {
                debug!(
                    "Found VCP message with {} elevation cuts",
                    chunk_vcp.elevations.len()
                );

                if vcp.is_none() {
                    *vcp = Some(*chunk_vcp.clone());
                    *elevation_chunk_mapper =
                        Some(ElevationChunkMapper::new(vcp.as_ref().unwrap()));
                }

                result.vcp_number = Some(format!("VCP{}", chunk_vcp.header.pattern_type));
            }
            MessageContents::DigitalRadarData(radar) => {
                if let Some(time) = radar.header.date_time() {
                    radar_times.push(time);
                }

                let mut add_data_type = |data_type: &str| {
                    *data_type_counter.entry(data_type.to_string()).or_insert(0) += 1;
                };

                if radar.volume_data_block.is_some() {
                    add_data_type("Volume");
                }

                if radar.elevation_data_block.is_some() {
                    add_data_type("Elevation");
                }

                if radar.radial_data_block.is_some() {
                    add_data_type("Radial");
                }

                if radar.reflectivity_data_block.is_some() {
                    add_data_type("Reflectivity");
                }

                if radar.velocity_data_block.is_some() {
                    add_data_type("Velocity");
                }

                if radar.spectrum_width_data_block.is_some() {
                    add_data_type("Spectrum Width");
                }

                if radar.differential_reflectivity_data_block.is_some() {
                    add_data_type("Differential Reflectivity");
                }

                if radar.differential_phase_data_block.is_some() {
                    add_data_type("Differential Phase");
                }

                if radar.correlation_coefficient_data_block.is_some() {
                    add_data_type("Correlation Coefficient");
                }

                if radar.specific_diff_phase_data_block.is_some() {
                    add_data_type("Specific Differential Phase");
                }

                result
                    .elevation_numbers
                    .insert(radar.header.elevation_number);
                if let Some(volume) = &radar.volume_data_block {
                    result.vcp_number =
                        Some(format!("VCP{}", volume.volume_coverage_pattern_number));
                }
            }
            _ => {}
        }
    }

    if !radar_times.is_empty() {
        radar_times.sort();
        result.earliest_message_time = Some(radar_times[0]);
        result.latest_message_time = Some(radar_times[radar_times.len() - 1]);

        if let (Some(earliest), Some(latest)) =
            (result.earliest_message_time, result.latest_message_time)
        {
            let duration = latest.signed_duration_since(earliest);
            result.scan_time = Some(duration.num_milliseconds() as f64 / 1000.0);

            let proc_duration = chunk_id.date_time().unwrap().signed_duration_since(latest);
            result.processing_time = Some(proc_duration.num_milliseconds() as f64 / 1000.0);
        }

        let start_azimuth = messages.iter().find_map(|msg| match msg.contents() {
            MessageContents::DigitalRadarData(radar) => Some(radar.header.azimuth_angle),
            _ => None,
        });

        let end_azimuth = messages.iter().rev().find_map(|msg| match msg.contents() {
            MessageContents::DigitalRadarData(radar) => Some(radar.header.azimuth_angle),
            _ => None,
        });

        if let (Some(start), Some(end)) = (start_azimuth, end_azimuth) {
            let range = if end > start {
                end - start
            } else {
                360.0 - (start - end)
            };

            result.azimuth_range = Some(range as f64);
        }
    }

    result.message_types = message_type_counter
        .keys()
        .cloned()
        .map(|msg_type| format!("{:?}", msg_type))
        .collect();

    result.data_types = data_type_counter
        .keys()
        .cloned()
        .map(|data_type| format!("{}", data_type))
        .collect();

    if let (Some(sequence), Some(vcp), Some(elevation_chunk_mapper)) =
        (chunk_id.sequence(), vcp, elevation_chunk_mapper)
    {
        let elevation = elevation_chunk_mapper
            .get_sequence_elevation_number(sequence)
            .and_then(|elevation_number| vcp.elevations.get(elevation_number - 1));

        if let Some(elevation) = elevation {
            result.matched_to_vcp = true;
            result.elevation_angle = Some(elevation.elevation_angle_degrees());
            result.channel_configuration = Some(format!("{:?}", elevation.channel_configuration()));
            result.waveform_type = Some(format!("{:?}", elevation.waveform_type()));
            result.super_resolution = Some(format!(
                "{:?}",
                elevation.super_resolution_control_half_degree_azimuth()
            ));
            result.azimuth_rate = Some(elevation.azimuth_rate_degrees_per_second());
        }
    }

    Ok(result)
}

/// Write a CSV row with chunk analysis data
fn write_csv_row(
    file: &mut File,
    chunk_name: String,
    modified_time: Option<DateTime<Utc>>,
    time_diff: String,
    mut analysis: ChunkAnalysis,
) -> Result<()> {
    analysis.message_types.sort();
    analysis.data_types.sort();

    writeln!(
        file,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        chunk_name,
        modified_time
            .map(|time| time.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
            .unwrap_or_else(|| "".to_string()),
        time_diff,
        analysis.message_types.join(";"),
        analysis.data_types.join(";"),
        analysis
            .earliest_message_time
            .map(|time| time.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
            .unwrap_or_else(|| "".to_string()),
        analysis
            .latest_message_time
            .map(|time| time.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
            .unwrap_or_else(|| "".to_string()),
        analysis
            .scan_time
            .map(|t| format!("{:.3}", t))
            .unwrap_or_else(|| "".to_string()),
        analysis
            .processing_time
            .map(|t| format!("{:.3}", t))
            .unwrap_or_else(|| "".to_string()),
        analysis
            .elevation_numbers
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(";"),
        analysis
            .elevation_angle
            .map(|e| format!("{:.2}", e))
            .unwrap_or_else(|| "".to_string()),
        analysis
            .matched_to_vcp
            .then(|| "Yes")
            .unwrap_or_else(|| "No"),
        analysis
            .azimuth_range
            .map(|a| format!("{:.2}", a))
            .unwrap_or_else(|| "".to_string()),
        analysis.vcp_number.unwrap_or_else(|| "".to_string()),
        analysis
            .channel_configuration
            .unwrap_or_else(|| "".to_string()),
        analysis.waveform_type.unwrap_or_else(|| "".to_string()),
        analysis.super_resolution.unwrap_or_else(|| "".to_string()),
        analysis
            .azimuth_rate
            .map(|a| format!("{:.2}", a))
            .unwrap_or_else(|| "".to_string())
    )?;

    Ok(())
}
