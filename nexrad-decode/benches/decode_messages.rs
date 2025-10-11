use std::{hint::black_box, io::Cursor, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use nexrad_data::volume;
use nexrad_decode::messages::decode_messages;

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

fn benchmark_decode_messages(c: &mut Criterion) {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());
    let mut records = volume.records().into_iter().take(2);

    let mut first_record = records.next().expect("first record exists");
    first_record = first_record
        .decompress()
        .expect("decompresses first record");

    let record1_data = first_record.data().to_vec();

    let mut second_record = records.next().expect("second record exists");
    if second_record.compressed() {
        second_record = second_record
            .decompress()
            .expect("decompresses second record");
    }

    let record2_data = second_record.data().to_vec();

    let mut group = c.benchmark_group("decode_messages");
    group
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .sample_size(200)
        .noise_threshold(0.05)
        .significance_level(0.02);

    group.bench_function("record_0", |b| {
        b.iter_batched(
            || record1_data.clone(),
            |data| {
                let mut reader = Cursor::new(black_box(&data));
                decode_messages(&mut reader).expect("decodes successfully")
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("record_1", |b| {
        b.iter_batched(
            || record2_data.clone(),
            |data| {
                let mut reader = Cursor::new(black_box(&data));
                decode_messages(&mut reader).expect("decodes successfully")
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, benchmark_decode_messages);
criterion_main!(benches);
