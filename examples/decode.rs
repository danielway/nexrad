//! examples/decode
//!
//! This example loads a data file and decodes it.
//!
//! Usage: cargo run --example decode -- <file>
//!

#![cfg(all(feature = "decompress"))]

use std::env;
use std::io::Cursor;

use nexrad::decompress::decompress_and_decode_archive2_file;
use nexrad::file::is_compressed;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: cargo run --example decode -- <file>");
    }

    let file_name = &args[1];
    let mut file = std::fs::read(file_name).expect("file exists");
    let mut reader = Cursor::new(file.as_slice());

    println!(
        "Loaded {} file of size {} bytes.",
        if is_compressed(reader.get_ref()) {
            "compressed"
        } else {
            "decompressed"
        },
        file.len()
    );

    if is_compressed(file.as_slice()) {
        // todo
        // file = decompress_file(&file)?;
        println!("Decompressed file data size (bytes): {}", file.len());
    }

    let decoded_file = decompress_and_decode_archive2_file(&mut reader, file.len() as u64)?;
    println!("Decoded file: {:?}", decoded_file);

    Ok(())
}
