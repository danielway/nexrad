//! Decode a NEXRAD volume file and print a summary.
//!
//! This example demonstrates using the facade API to load radar data
//! and inspect its structure.
//!
//! Run with:
//! ```bash
//! cargo run --example decode_summary -- tests/fixtures/convective/KDMX20220305_232324.bin
//! ```

use std::env;

fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("tests/fixtures/convective/KDMX20220305_232324.bin");

    println!("Loading volume from: {}", path);
    let volume = nexrad::load_file(path)?;

    println!("\n=== Volume Summary ===");
    println!("VCP: {}", volume.coverage_pattern_number());
    println!("Sweeps: {}", volume.sweeps().len());

    let vcp = volume.coverage_pattern();
    println!("\n=== VCP Configuration ===");
    println!("Version: {}", vcp.version());
    println!(
        "Doppler Resolution: {} m/s",
        vcp.doppler_velocity_resolution()
    );
    println!("SAILS: {} ({} cuts)", vcp.sails_enabled(), vcp.sails_cuts());
    println!("MRLE: {} ({} cuts)", vcp.mrle_enabled(), vcp.mrle_cuts());
    println!("MPDA: {}", vcp.mpda_enabled());

    println!("\n=== Elevation Cuts ===");
    for (i, cut) in vcp.elevation_cuts().iter().enumerate() {
        println!(
            "  Cut {}: {:.1} deg, {:?}",
            i + 1,
            cut.elevation_angle_degrees(),
            cut.waveform_type()
        );
    }

    println!("\n=== Sweep Details ===");
    for (i, sweep) in volume.sweeps().iter().enumerate() {
        let radials = sweep.radials();
        let first = radials.first();

        let (refl_count, vel_count) = radials.iter().fold((0, 0), |(r, v), radial| {
            (
                r + if radial.reflectivity().is_some() {
                    1
                } else {
                    0
                },
                v + if radial.velocity().is_some() { 1 } else { 0 },
            )
        });

        println!(
            "  Sweep {}: elev {}, {} radials, {:.1} deg start, refl={}, vel={}",
            i + 1,
            sweep.elevation_number(),
            radials.len(),
            first.map(|r| r.azimuth_angle_degrees()).unwrap_or(0.0),
            refl_count,
            vel_count
        );
    }

    Ok(())
}
