//!
//! examples/render
//!
//! This example loads a data file and renders it according to various options.
//!

use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};

const IMAGE_SIZE: usize = 1024;

const BELOW_THRESHOLD: f32 = 999.0;
const MOMENT_FOLDED: f32 = 998.0;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: cargo run --example decode -- <file> [product] [elevationIndex]");
    }

    let file_name = &args[1];
    let mut file = std::fs::read(file_name).expect("file exists");

    let mut requested_product = "ref";
    if args.len() > 2 {
        requested_product = &args[2];
    }

    let mut requested_elevation_index = 0;
    if args.len() > 3 {
        requested_elevation_index = args[3].parse::<usize>().unwrap();
    }

    println!(
        "Loaded {} file of size {} bytes.",
        if is_compressed(file.as_slice()) {
            "compressed"
        } else {
            "decompressed"
        },
        file.len()
    );

    if is_compressed(file.as_slice()) {
        file = decompress_file(&file).expect("decompresses file");
        println!("Decompressed file data size (bytes): {}", file.len());
    }

    let decoded = decode_file(&file).expect("decodes file");
    println!(
        "Decoded file with {} elevations.",
        decoded.elevation_scans().len()
    );

    println!(
        "Rendering {} product at elevation index {}.",
        requested_product, requested_elevation_index
    );
    let rendered_image = render_ppm_image(&decoded, requested_elevation_index, requested_product);

    let file_name = format!(
        "render_{}_{}.ppm",
        requested_product, requested_elevation_index
    );
    println!("Writing rendered image to {}", file_name);
    write_ppm_image(&file_name, IMAGE_SIZE, rendered_image).expect("write file");
}

fn render_ppm_image(
    decoded: &DataFile,
    requested_elevation_index: usize,
    requested_product: &str,
) -> Vec<(u8, u8, u8)> {
    let mut pixel_data = vec![(0, 0, 0); IMAGE_SIZE * IMAGE_SIZE];

    let center = IMAGE_SIZE / 2;
    let px_per_km = IMAGE_SIZE / 2 / 460;

    let mut elevation_scans: Vec<_> = decoded.elevation_scans().iter().collect();
    elevation_scans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let (_, radials) = elevation_scans[requested_elevation_index];

    let radial = radials.iter().next().unwrap();
    let radial_reflectivity = radial.reflectivity_data().unwrap().data();

    let moment_range = radial_reflectivity.data_moment_range();
    let first_gate_px = moment_range as f32 / 1000.0 * px_per_km as f32;

    let gate_interval_km = radial_reflectivity.data_moment_range_sample_interval() as f64 / 1000.0;
    let gate_width_px = gate_interval_km * px_per_km as f64;

    for radial in radials {
        let mut azimuth_angle = radial.header().azm() - 90.0;
        if azimuth_angle < 0.0 {
            azimuth_angle = 360.0 + azimuth_angle;
        }

        let azimuth_spacing = radial.header().azm_res() as f32;

        let mut azimuth = azimuth_angle.floor();
        if (azimuth_angle + azimuth_spacing).floor() > azimuth {
            azimuth += azimuth_spacing;
        }

        let start_angle = azimuth * (PI / 180.0);

        let mut distance = first_gate_px;

        let data_moment = match requested_product {
            "ref" => radial.reflectivity_data().unwrap(),
            "vel" => radial.velocity_data().unwrap(),
            "sw" => radial.sw_data().unwrap(),
            "phi" => radial.phi_data().unwrap(),
            "rho" => radial.rho_data().unwrap(),
            "zdr" => radial.zdr_data().unwrap(),
            "cfp" => radial.cfp_data().unwrap(),
            _ => panic!("Unexpected product: {}", requested_product),
        };

        let mut raw_gates: Vec<u16> =
            vec![0; data_moment.data().number_data_moment_gates() as usize];

        assert_eq!(data_moment.data().data_word_size(), 8);
        for (i, v) in data_moment.moment_data().iter().enumerate() {
            raw_gates[i] = *v;
        }

        let mut scaled_gates: Vec<f32> = Vec::new();
        for raw_gate in raw_gates {
            if raw_gate == 0 {
                scaled_gates.push(BELOW_THRESHOLD);
            } else if raw_gate == 1 {
                scaled_gates.push(MOMENT_FOLDED);
            } else {
                let scale = data_moment.data().scale();
                let offset = data_moment.data().offset();

                let scaled_gate = if scale == 0.0 {
                    raw_gate as f32
                } else {
                    (raw_gate as f32 - offset) / scale
                };

                scaled_gates.push(scaled_gate);
            }
        }

        for scaled_gate in scaled_gates {
            if scaled_gate != BELOW_THRESHOLD {
                let angle_cos = start_angle.cos();
                let angle_sin = start_angle.sin();

                let pixel_x = (center as f32 + angle_cos * distance).round() as usize;
                let pixel_y = (center as f32 + angle_sin * distance).round() as usize;

                pixel_data[pixel_y * IMAGE_SIZE + pixel_x] =
                    if scaled_gate < 5.0 || scaled_gate == BELOW_THRESHOLD {
                        (0, 0, 0)
                    } else if scaled_gate >= 5.0 && scaled_gate < 10.0 {
                        (0x40, 0xe8, 0xe3)
                    } else if scaled_gate >= 10.0 && scaled_gate < 15.0 {
                        (0x26, 0xa4, 0xfa)
                    } else if scaled_gate >= 15.0 && scaled_gate < 20.0 {
                        (0x00, 0x30, 0xed)
                    } else if scaled_gate >= 20.0 && scaled_gate < 25.0 {
                        (0x49, 0xfb, 0x3e)
                    } else if scaled_gate >= 25.0 && scaled_gate < 30.0 {
                        (0x36, 0xc2, 0x2e)
                    } else if scaled_gate >= 30.0 && scaled_gate < 35.0 {
                        (0x27, 0x8c, 0x1e)
                    } else if scaled_gate >= 35.0 && scaled_gate < 40.0 {
                        (0xfe, 0xf5, 0x43)
                    } else if scaled_gate >= 40.0 && scaled_gate < 45.0 {
                        (0xeb, 0xb4, 0x33)
                    } else if scaled_gate >= 45.0 && scaled_gate < 50.0 {
                        (0xf6, 0x95, 0x2e)
                    } else if scaled_gate >= 50.0 && scaled_gate < 55.0 {
                        (0xf8, 0x0a, 0x26)
                    } else if scaled_gate >= 55.0 && scaled_gate < 60.0 {
                        (0xcb, 0x05, 0x16)
                    } else if scaled_gate >= 60.0 && scaled_gate < 65.0 {
                        (0xa9, 0x08, 0x13)
                    } else if scaled_gate >= 65.0 && scaled_gate < 70.0 {
                        (0xee, 0x34, 0xfa)
                    } else {
                        (0xff, 0xff, 0xFF)
                    };
            }

            distance += gate_width_px as f32;
            azimuth += azimuth_spacing;
        }
    }

    pixel_data
}

fn write_ppm_image(file: &str, width: usize, data: Vec<(u8, u8, u8)>) -> io::Result<()> {
    let mut file = File::create(file)?;

    file.write_all(format!("P3\n{} {}\n255\n", width, width).as_bytes())?;
    for (r, g, b) in data {
        file.write_all(format!("{} {} {}\n", r, g, b).as_bytes())?;
    }

    Ok(())
}
