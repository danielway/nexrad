//!
//! examples/message_header
//!
//! This example loads and decodes a message header from a file before printing its contents.
//!
//! Usage: cargo run --example message_header
//!

use nexrad_decode::messages::decode_message_header;
use std::fs;

fn main() {
    let file = fs::read("examples/data/message_header").expect("file exists");
    let mut reader = std::io::Cursor::new(file.as_slice());

    let message_header = decode_message_header(&mut reader).unwrap();
    println!("Decoded message header: {:?}", message_header);
}
