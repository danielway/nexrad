//! examples/decompress
//!
//! This example loads a file, decompresses it, and prints its size.
//!
//! Usage: cargo run --example decompress -- <file>
//!

#![cfg(all(feature = "decompress"))]

use nexrad::file::is_compressed;
use nexrad::result::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: cargo run --example decompress -- <file>");
    }

    let file_name = &args[1];
    let mut file = std::fs::read(file_name).expect("file exists");

    if !is_compressed(file.as_slice()) {
        panic!("File is not compressed.");
    }

    println!("Data file size (bytes): {}", file.len());
    println!(
        "Data file is compressed: {}",
        is_compressed(file.as_slice())
    );

    file = decompress_file(&file)?;
    println!("Decompressed file data size (bytes): {}", file.len());
    println!(
        "Decompressed file data is compressed: {}",
        is_compressed(file.as_slice())
    );

    Ok(())
}
