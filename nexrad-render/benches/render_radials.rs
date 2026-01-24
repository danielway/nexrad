use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};
use nexrad_decode::messages::MessageContents;
use nexrad_model::data::Radial;
use nexrad_render::{get_nws_reflectivity_scale, render_radials, Product, RenderOptions};

const VOLUME_FILE: &[u8] =
    include_bytes!("../../tests/fixtures/convective/KDMX20220305_232324.bin");

/// Extract radials for a specific elevation number from a volume file.
fn extract_radials(data: &[u8], elevation_number: u8) -> Vec<Radial> {
    let archive = nexrad_data::volume::File::new(data.to_vec());
    let mut radials = Vec::new();

    for mut record in archive.records().expect("records") {
        if record.compressed() {
            record = record.decompress().expect("decompresses successfully");
        }

        for message in record.messages().expect("messages are valid") {
            if let MessageContents::DigitalRadarData(message) = message.contents() {
                if message.header().elevation_number() == elevation_number {
                    radials.push(message.radial().expect("radial is valid"));
                }
            }
        }
    }

    radials
}

fn benchmark_render_radials(c: &mut Criterion) {
    // Extract radials for the 0.5 degree sweep (elevation 1)
    let radials = extract_radials(VOLUME_FILE, 1);
    let color_scale = get_nws_reflectivity_scale();

    let mut group = c.benchmark_group("render_radials");
    group
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);

    group.bench_function("reflectivity_800x800", |b| {
        let options = RenderOptions::new(800, 800);
        b.iter(|| {
            render_radials(
                black_box(&radials),
                Product::Reflectivity,
                &color_scale,
                &options,
            )
            .expect("renders successfully")
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_render_radials);
criterion_main!(benches);
