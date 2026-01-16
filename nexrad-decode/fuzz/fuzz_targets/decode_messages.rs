#![no_main]

use libfuzzer_sys::fuzz_target;
use nexrad_decode::messages::decode_messages;

fuzz_target!(|data: &[u8]| {
    // Fuzz the message decoding entry point
    // This should never panic - only return Ok or Err
    let _ = decode_messages(data);
});
