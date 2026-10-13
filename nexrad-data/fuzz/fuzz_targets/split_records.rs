#![no_main]

use libfuzzer_sys::fuzz_target;
use nexrad_data::volume::File;

fuzz_target!(|data: &[u8]| {
    // Fuzz volume header parsing and record splitting through the public volume API.
    // This should never panic - only return Ok or Err
    let file = File::new(data.to_vec());
    let _ = file.header();
    let _ = file.records();
});
