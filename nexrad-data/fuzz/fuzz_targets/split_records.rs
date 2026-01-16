#![no_main]

use libfuzzer_sys::fuzz_target;
use nexrad_data::volume::split_compressed_records;

fuzz_target!(|data: &[u8]| {
    // Fuzz the record splitting entry point
    // This should never panic - only return Ok or Err
    let _ = split_compressed_records(data);
});
