#![no_main]

use libfuzzer_sys::fuzz_target;
use nexrad_data::volume::Record;

fuzz_target!(|data: &[u8]| {
    // Fuzz the record decompression path
    // This should never panic - only return Ok or Err
    let record = Record::new(data.to_vec());
    if record.compressed() {
        let _ = record.decompress();
    }
});
