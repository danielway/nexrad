//! NEXRAD Sweep Timing Analysis
//!
//! Analyzes radial-level timestamps across diverse archive files to understand:
//! - Sweep durations
//! - Inter-sweep gaps
//! - Inter-volume gaps
//! - Relationship between VCP parameters and timing
//!
//! Run with:
//! ```bash
//! cargo run --release --example timing_analysis 2>/dev/null
//! ```

use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use std::path::Path;

/// Files to download from AWS for analysis - chosen for geographic, temporal,
/// and meteorological diversity
const DOWNLOADS: &[(&str, &str)] = &[
    // === Clear air scenarios ===
    // KTLX (Oklahoma) - clear air, winter night
    ("KTLX", "2023-01-15"),
    // KLIX (New Orleans) - clear air, summer
    ("KLIX", "2023-07-20"),
    // KPUX (Pueblo, CO) - clear air, high plains
    ("KPUX", "2022-12-01"),
    // === Convective/severe weather ===
    // KTLX (Oklahoma) - May severe weather
    ("KTLX", "2024-05-06"),
    // KFWS (Dallas) - severe storms
    ("KFWS", "2023-06-15"),
    // KILX (Lincoln, IL) - Midwest storms
    ("KILX", "2024-04-15"),
    // === Hurricane/tropical ===
    // KCRP (Corpus Christi) - Hurricane Harvey
    ("KCRP", "2017-08-26"),
    // KMLB (Melbourne, FL) - Hurricane Ian approach
    ("KMLB", "2022-09-28"),
    // === Winter weather ===
    // KBUF (Buffalo) - lake effect snow
    ("KBUF", "2022-12-24"),
    // KMKX (Milwaukee) - winter storm
    ("KMKX", "2023-02-22"),
    // === Widespread precipitation ===
    // KLWX (Sterling, VA/DC) - rain
    ("KLWX", "2023-03-10"),
    // KGRR (Grand Rapids) - rain
    ("KGRR", "2024-06-20"),
    // === Older data for temporal comparison ===
    // KTLX 2013 - tornado outbreak (already in downloads)
    // KTLX 2019 - already in downloads

    // === Geographic extremes ===
    // PHKI (Hawaii) - tropical Pacific
    ("PHKI", "2023-08-15"),
    // PAPD (Fairbanks, AK) - Alaska
    ("PAPD", "2023-06-15"),
    // TJUA (San Juan, PR) - Caribbean
    ("TJUA", "2023-09-10"),
];

#[derive(Debug, Clone)]
struct SweepTiming {
    elevation_number: u8,
    elevation_angle_deg: f32,
    radial_count: usize,
    azimuth_spacing_deg: f32,
    first_radial_time: DateTime<Utc>,
    last_radial_time: DateTime<Utc>,
    duration_ms: i64,
    has_reflectivity: bool,
    has_velocity: bool,
    has_dual_pol: bool,
}

#[derive(Debug, Clone)]
struct VCPCutInfo {
    elevation_angle_deg: f64,
    waveform_type: String,
    azimuth_rate_dps: f64,
    super_res_half_deg: bool,
    is_sails: bool,
    is_mrle: bool,
    prf_number: u8,
    prf_pulse_count: u16,
}

#[derive(Debug, Clone)]
struct VolumeTiming {
    filename: String,
    site: String,
    vcp_number: String,
    vcp_version: u8,
    sails_enabled: bool,
    sails_cuts: u8,
    mrle_enabled: bool,
    mrle_cuts: u8,
    mpda_enabled: bool,
    total_sweeps: usize,
    sweep_timings: Vec<SweepTiming>,
    inter_sweep_gaps_ms: Vec<i64>,
    vcp_cuts: Vec<VCPCutInfo>,
    volume_start: DateTime<Utc>,
    volume_end: DateTime<Utc>,
    volume_duration_ms: i64,
}

fn analyze_scan(
    scan: &nexrad::model::data::Scan,
    filename: &str,
    site: &str,
) -> Option<VolumeTiming> {
    let vcp = scan.coverage_pattern();
    let sweeps = scan.sweeps();

    if sweeps.is_empty() {
        return None;
    }

    let mut sweep_timings: Vec<SweepTiming> = Vec::new();

    for sweep in sweeps {
        let radials = sweep.radials();
        if radials.is_empty() {
            continue;
        }

        let times: Vec<DateTime<Utc>> =
            radials.iter().filter_map(|r| r.collection_time()).collect();

        if times.is_empty() {
            continue;
        }

        let first_time = *times.iter().min().unwrap_or(&times[0]);
        let last_time = *times.iter().max().unwrap_or(&times[0]);
        let duration_ms = (last_time - first_time).num_milliseconds();

        let first_radial = radials.first().unwrap();

        let has_dual_pol = radials
            .iter()
            .any(|r| r.differential_reflectivity().is_some());

        sweep_timings.push(SweepTiming {
            elevation_number: sweep.elevation_number(),
            elevation_angle_deg: sweep.elevation_angle_degrees().unwrap_or(0.0),
            radial_count: radials.len(),
            azimuth_spacing_deg: first_radial.azimuth_spacing_degrees(),
            first_radial_time: first_time,
            last_radial_time: last_time,
            duration_ms,
            has_reflectivity: radials.iter().any(|r| r.reflectivity().is_some()),
            has_velocity: radials.iter().any(|r| r.velocity().is_some()),
            has_dual_pol,
        });
    }

    if sweep_timings.is_empty() {
        return None;
    }

    // Sort by first radial time to compute gaps correctly
    sweep_timings.sort_by_key(|s| s.first_radial_time);

    let inter_sweep_gaps_ms: Vec<i64> = sweep_timings
        .windows(2)
        .map(|w| (w[1].first_radial_time - w[0].last_radial_time).num_milliseconds())
        .collect();

    let volume_start = sweep_timings.first().unwrap().first_radial_time;
    let volume_end = sweep_timings.last().unwrap().last_radial_time;
    let volume_duration_ms = (volume_end - volume_start).num_milliseconds();

    let vcp_cuts: Vec<VCPCutInfo> = vcp
        .elevation_cuts()
        .iter()
        .map(|cut| VCPCutInfo {
            elevation_angle_deg: cut.elevation_angle_degrees(),
            waveform_type: format!("{:?}", cut.waveform_type()),
            azimuth_rate_dps: cut.azimuth_rate_degrees_per_second(),
            super_res_half_deg: cut.super_resolution_half_degree_azimuth(),
            is_sails: cut.is_sails_cut(),
            is_mrle: cut.is_mrle_cut(),
            prf_number: cut.surveillance_prf_number(),
            prf_pulse_count: cut.surveillance_prf_pulse_count(),
        })
        .collect();

    Some(VolumeTiming {
        filename: filename.to_string(),
        site: site.to_string(),
        vcp_number: format!("{}", scan.coverage_pattern_number()),
        vcp_version: vcp.version(),
        sails_enabled: vcp.sails_enabled(),
        sails_cuts: vcp.sails_cuts(),
        mrle_enabled: vcp.mrle_enabled(),
        mrle_cuts: vcp.mrle_cuts(),
        mpda_enabled: vcp.mpda_enabled(),
        total_sweeps: sweep_timings.len(),
        sweep_timings,
        inter_sweep_gaps_ms,
        vcp_cuts,
        volume_start,
        volume_end,
        volume_duration_ms,
    })
}

fn print_json_output(results: &[VolumeTiming]) {
    println!("[");
    for (vi, vol) in results.iter().enumerate() {
        println!("  {{");
        println!("    \"filename\": \"{}\",", vol.filename);
        println!("    \"site\": \"{}\",", vol.site);
        println!("    \"vcp_number\": \"{}\",", vol.vcp_number);
        println!("    \"vcp_version\": {},", vol.vcp_version);
        println!("    \"sails_enabled\": {},", vol.sails_enabled);
        println!("    \"sails_cuts\": {},", vol.sails_cuts);
        println!("    \"mrle_enabled\": {},", vol.mrle_enabled);
        println!("    \"mrle_cuts\": {},", vol.mrle_cuts);
        println!("    \"mpda_enabled\": {},", vol.mpda_enabled);
        println!("    \"total_sweeps\": {},", vol.total_sweeps);
        println!("    \"volume_start\": \"{}\",", vol.volume_start);
        println!("    \"volume_end\": \"{}\",", vol.volume_end);
        println!("    \"volume_duration_ms\": {},", vol.volume_duration_ms);

        // VCP cuts
        println!("    \"vcp_cuts\": [");
        for (ci, cut) in vol.vcp_cuts.iter().enumerate() {
            println!("      {{");
            println!(
                "        \"elevation_angle_deg\": {:.2},",
                cut.elevation_angle_deg
            );
            println!("        \"waveform_type\": \"{}\",", cut.waveform_type);
            println!("        \"azimuth_rate_dps\": {:.3},", cut.azimuth_rate_dps);
            println!(
                "        \"super_res_half_deg\": {},",
                cut.super_res_half_deg
            );
            println!("        \"is_sails\": {},", cut.is_sails);
            println!("        \"is_mrle\": {},", cut.is_mrle);
            println!("        \"prf_number\": {},", cut.prf_number);
            println!("        \"prf_pulse_count\": {}", cut.prf_pulse_count);
            if ci < vol.vcp_cuts.len() - 1 {
                println!("      }},");
            } else {
                println!("      }}");
            }
        }
        println!("    ],");

        // Sweep timings
        println!("    \"sweep_timings\": [");
        for (si, sweep) in vol.sweep_timings.iter().enumerate() {
            println!("      {{");
            println!("        \"elevation_number\": {},", sweep.elevation_number);
            println!(
                "        \"elevation_angle_deg\": {:.2},",
                sweep.elevation_angle_deg
            );
            println!("        \"radial_count\": {},", sweep.radial_count);
            println!(
                "        \"azimuth_spacing_deg\": {:.1},",
                sweep.azimuth_spacing_deg
            );
            println!(
                "        \"first_radial_time\": \"{}\",",
                sweep.first_radial_time
            );
            println!(
                "        \"last_radial_time\": \"{}\",",
                sweep.last_radial_time
            );
            println!("        \"duration_ms\": {},", sweep.duration_ms);
            println!("        \"has_reflectivity\": {},", sweep.has_reflectivity);
            println!("        \"has_velocity\": {},", sweep.has_velocity);
            println!("        \"has_dual_pol\": {}", sweep.has_dual_pol);
            if si < vol.sweep_timings.len() - 1 {
                println!("      }},");
            } else {
                println!("      }}");
            }
        }
        println!("    ],");

        // Inter-sweep gaps
        println!("    \"inter_sweep_gaps_ms\": [");
        for (gi, gap) in vol.inter_sweep_gaps_ms.iter().enumerate() {
            if gi < vol.inter_sweep_gaps_ms.len() - 1 {
                println!("      {},", gap);
            } else {
                println!("      {}", gap);
            }
        }
        println!("    ]");

        if vi < results.len() - 1 {
            println!("  }},");
        } else {
            println!("  }}");
        }
    }
    println!("]");
}

fn summarize_results(results: &[VolumeTiming]) {
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("NEXRAD SWEEP TIMING ANALYSIS SUMMARY");
    eprintln!("{}", "=".repeat(80));
    eprintln!("Total volumes analyzed: {}", results.len());

    // Group by VCP
    let mut by_vcp: HashMap<String, Vec<&VolumeTiming>> = HashMap::new();
    for vol in results {
        by_vcp.entry(vol.vcp_number.clone()).or_default().push(vol);
    }

    for (vcp, vols) in &by_vcp {
        eprintln!("\n--- VCP {} ({} volumes) ---", vcp, vols.len());

        // Volume durations
        let vol_durations: Vec<f64> = vols
            .iter()
            .map(|v| v.volume_duration_ms as f64 / 1000.0)
            .collect();
        eprintln!(
            "  Volume duration: min={:.1}s, max={:.1}s, mean={:.1}s",
            vol_durations.iter().cloned().fold(f64::INFINITY, f64::min),
            vol_durations
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max),
            vol_durations.iter().sum::<f64>() / vol_durations.len() as f64
        );

        // Sweep counts
        let sweep_counts: Vec<usize> = vols.iter().map(|v| v.total_sweeps).collect();
        eprintln!(
            "  Sweep count: min={}, max={}",
            sweep_counts.iter().min().unwrap_or(&0),
            sweep_counts.iter().max().unwrap_or(&0)
        );

        // Sweep durations
        let all_sweep_durations: Vec<f64> = vols
            .iter()
            .flat_map(|v| {
                v.sweep_timings
                    .iter()
                    .map(|s| s.duration_ms as f64 / 1000.0)
            })
            .collect();
        if !all_sweep_durations.is_empty() {
            eprintln!(
                "  Sweep duration: min={:.2}s, max={:.2}s, mean={:.2}s",
                all_sweep_durations
                    .iter()
                    .cloned()
                    .fold(f64::INFINITY, f64::min),
                all_sweep_durations
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max),
                all_sweep_durations.iter().sum::<f64>() / all_sweep_durations.len() as f64
            );
        }

        // Inter-sweep gaps
        let all_gaps: Vec<f64> = vols
            .iter()
            .flat_map(|v| v.inter_sweep_gaps_ms.iter().map(|g| *g as f64 / 1000.0))
            .collect();
        if !all_gaps.is_empty() {
            eprintln!(
                "  Inter-sweep gap: min={:.2}s, max={:.2}s, mean={:.2}s",
                all_gaps.iter().cloned().fold(f64::INFINITY, f64::min),
                all_gaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                all_gaps.iter().sum::<f64>() / all_gaps.len() as f64
            );
        }
    }
}

#[tokio::main]
async fn main() -> nexrad::Result<()> {
    let mut results: Vec<VolumeTiming> = Vec::new();

    // Phase 1: Analyze existing local files
    eprintln!("=== Phase 1: Analyzing existing local files ===");
    let downloads_dir = Path::new("downloads");

    if downloads_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(downloads_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.') && !name.ends_with(".md") && !name.ends_with("_MDM")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy().to_string();

            // Extract site from filename (first 4 chars)
            let site = if filename.len() >= 4 {
                &filename[..4]
            } else {
                "UNKN"
            };

            eprintln!("  Loading: {}", filename);
            match nexrad::load_file(&path) {
                Ok(scan) => {
                    if let Some(timing) = analyze_scan(&scan, &filename, site) {
                        eprintln!(
                            "    VCP={}, sweeps={}, duration={:.1}s",
                            timing.vcp_number,
                            timing.total_sweeps,
                            timing.volume_duration_ms as f64 / 1000.0
                        );
                        results.push(timing);
                    } else {
                        eprintln!("    Skipped (no valid timing data)");
                    }
                }
                Err(e) => {
                    eprintln!("    Error: {}", e);
                }
            }
        }
    }

    // Phase 2: Download additional files from AWS
    eprintln!("\n=== Phase 2: Downloading additional files from AWS ===");

    for (site, date_str) in DOWNLOADS {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        eprintln!("  Listing scans for {} on {}...", site, date_str);

        match nexrad::list_scans(site, date).await {
            Ok(scans) => {
                if scans.is_empty() {
                    eprintln!("    No scans available");
                    continue;
                }

                // Pick 2 scans: one from early in the day, one from later
                // This helps capture different VCP modes that might be active
                let indices = if scans.len() >= 4 {
                    vec![scans.len() / 4, scans.len() * 3 / 4]
                } else {
                    vec![0]
                };

                for idx in indices {
                    let scan_id = &scans[idx];
                    let name = scan_id.name().to_string();
                    eprintln!("    Downloading: {}", name);

                    match nexrad::download(scan_id.clone()).await {
                        Ok(scan) => {
                            if let Some(timing) = analyze_scan(&scan, &name, site) {
                                eprintln!(
                                    "      VCP={}, sweeps={}, duration={:.1}s",
                                    timing.vcp_number,
                                    timing.total_sweeps,
                                    timing.volume_duration_ms as f64 / 1000.0
                                );
                                results.push(timing);
                            } else {
                                eprintln!("      Skipped (no valid timing data)");
                            }
                        }
                        Err(e) => {
                            eprintln!("      Error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("    Error listing: {}", e);
            }
        }
    }

    // Print summary to stderr
    summarize_results(&results);

    // Print full JSON to stdout
    print_json_output(&results);

    Ok(())
}
