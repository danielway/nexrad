# NEXRAD

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/rust_ci.yml)
[![Rust CD](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/rust_cd.yml)

Download and decode functions for NEXRAD radar data.

## Summary

The U.S. operates a network of 160 WSR-88D weather radars as part of the NEXRAD (Next Generation Radar) system. Its 
range and spatial resolution vary by data type, but it has a reflectivity range up to 460km and a base spatial 
resolution of 1km x 1deg. The data from this system moves through three levels of processing that generate "products"
which may be used for weather forecasting and research.

This library provides functions to download and decode NEXRAD Level II data from AWS uploaded in near real-time by NOAA.

## Downloading

The `download` feature may be enabled to download NEXRAD Level II data from AWS. For more information on this data
source, see the [Registry of Open Data](https://registry.opendata.aws/noaa-nexrad/)'s page. As the radar rotates or
"sweeps" it collects data which is aggregated into messages. The messages are broken into 5-minute "chunks" before being 
compressed and uploaded to AWS.

The data is organized by site and date. Here is an example of downloading the first file for April 6, 2023 from KDMX:
```rust
let site = "KDMX";
let date = NaiveDate::from_ymd_opt(2023, 4, 6).expect("is valid date");

let metas = list_files(site, &date).await?;
if let Some(meta) = metas.first() {
    println!("Downloading {}...", meta.identifier());
    let compressed_file = download_file(meta).await?;
    
    println!("Data file size (bytes): {}", compressed_file.len());
    println!("Data file is compressed: {}", is_compressed(compressed_file));
} else {
    println!("No files found for the specified date/site to download.");
}
```

In this example, `list_files` is being used to query which files are available for the specified site and date, and
`download_file` is used to download the contents of the first file. The downloaded file will need to be decompressed and
decoded before the data can be inspected.

## Decompression

Raw data files are compressed with bzip2 and must be decompressed prior to decoding. Here is an example of 
decompressing a file: 
```rust
let compressed_file = ...;

println!("Data file size (bytes): {}", compressed_file.data().len());
println!("Data file is compressed: {}", compressed_file.compressed());

let decompressed_file = decompress_file(&compressed_file)?;
println!("Decompressed data file size (bytes): {}", decompressed_file.data().len());
println!("Decompressed data file is compressed: {}", decompressed_file.compressed());
```

## Decoding

A decompressed data file consists of binary-encoded messages containing sweep data. Here is an example of decoding a 
file:
```rust
let data = ...;
let decoded = decode_data(&data)?;
println!("Decoded file: {:?}", decoded);
```

Data files contain a wide variety of data/messages, but you may only be interested in a particular subset. To reduce the
volume of data that needs to be stored or processed, you can apply a filter when decoding a file. For example:
```rust
// TODO
```

The decoded models are binary-serializable, so they can then be cached to disk for reuse later.
