//! examples/decode
//!
//! This example loads a data file and decodes it.
//!
//! Usage: cargo run --example decode -- <file>
//!

use std::env;

use nexrad::decode::decode_archive2_header;
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

    println!(
        "Loaded {} file of size {} bytes.",
        if is_compressed(file.as_slice()) {
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

    let decoded_archive2_header = decode_archive2_header(&mut file.as_slice())?;
    println!("Decoded Archive2Header: {:?}", decoded_archive2_header);

    Ok(())
}
