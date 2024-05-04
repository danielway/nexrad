//! examples/decode
//!
//! This example loads a data file and decodes it.
//!
//! Usage: cargo run --example decode -- <file or directory>
//!

#![cfg(all(feature = "decompress"))]

use std::io::Cursor;
use std::{env, fs};

use nexrad::decompress::decompress_and_decode_archive2_file;
use nexrad::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: cargo run --example decode -- <file or directory>");
    }

    let path = &args[1];
    let path_metadata = fs::metadata(path).expect("file exists");

    let file_names = if path_metadata.is_dir() {
        fs::read_dir(path)
            .expect("directory exists")
            .map(|entry| {
                let file_name = entry.expect("entry exists").file_name();
                format!(
                    "{}/{}",
                    path,
                    file_name.to_str().expect("file name is valid")
                )
            })
            // These don't seem to be Archive2 files, so skip them
            .filter(|file_name| {
                let is_mdm = file_name.ends_with("MDM");
                if is_mdm {
                    println!("Skipping non-Archive2 file: {}", file_name);
                }
                !is_mdm
            })
            .collect()
    } else {
        vec![path.to_string()]
    };

    println!("Loading {} file(s)...", file_names.len());
    for file_name in &file_names {
        let file = fs::read(file_name).expect("file exists");
        println!("Loaded file of size {} bytes.", file.len());

        let mut reader = Cursor::new(file.as_slice());
        let decoded_file = decompress_and_decode_archive2_file(&mut reader, file.len() as u64)?;
        println!("Decoded file: {:?}", decoded_file.header);
    }

    println!("Decoded {} files.", file_names.len());

    Ok(())
}
