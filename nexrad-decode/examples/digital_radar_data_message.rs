//!
//! examples/digital_radar_data_message
//!
//! This example loads and decodes a digital radar data message (type 31) from a file before
//! printing its contents.
//!
//! Usage: cargo run --example digital_radar_data_message
//!

use nexrad_decode::messages::digital_radar_data::decode_digital_radar_data;
use std::fs;

fn main() {
    let file = fs::read("examples/data/digital_radar_data_message").expect("file exists");
    let mut reader = std::io::Cursor::new(file.as_slice());

    let message = decode_digital_radar_data(&mut reader).unwrap();
    println!("Decoded digital radar data message: {:?}", message);

    #[cfg(feature = "nexrad-model")]
    println!("Decoded message radial model: {:?}", message.radial());
}
