use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use nexrad_data::volume::Record;
use nexrad_decode::messages::decode_messages;

const RECORD_00: &[u8] = include_bytes!("data/KDMX20220305_record_00.bin");
const RECORD_01: &[u8] = include_bytes!("data/KDMX20220305_record_01.bin");
const RECORD_37: &[u8] = include_bytes!("data/KDMX20220305_record_37.bin");

fn benchmark_decode_messages(c: &mut Criterion) {
    let record_00 = Record::new(RECORD_00.to_vec())
        .decompress()
        .expect("decompresses record 00");
    let record_00_data = record_00.data().to_vec();

    let record_01 = Record::new(RECORD_01.to_vec())
        .decompress()
        .expect("decompresses record 01");
    let record_01_data = record_01.data().to_vec();

    let record_37 = Record::new(RECORD_37.to_vec())
        .decompress()
        .expect("decompresses record 37");
    let record_37_data = record_37.data().to_vec();

    let mut group = c.benchmark_group("decode_messages");
    group
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .sample_size(200)
        .noise_threshold(0.05)
        .significance_level(0.02);

    group.bench_function("record_00", |b| {
        b.iter_batched(
            || record_00_data.clone(),
            |data| {
                decode_messages(black_box(&data)).expect("decodes successfully");
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("record_01", |b| {
        b.iter_batched(
            || record_01_data.clone(),
            |data| {
                decode_messages(black_box(&data)).expect("decodes successfully");
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("record_37", |b| {
        b.iter(|| {
            decode_messages(black_box(&record_37_data)).expect("decodes successfully");
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_decode_messages);
criterion_main!(benches);
