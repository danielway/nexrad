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

let metas = list_chunks(site, &date).await?;
if let Some(meta) = metas.first() {
    println!("Downloading {}...", meta.identifier());
    let compressed_chunk = download_chunk(meta).await?;
    
    println!("Chunk data size (bytes): {}", compressed_chunk.data().len());
    println!("Chunk data is compressed: {}", compressed_chunk.compressed());
} else {
    println!("No chunks found for the specified date/site to download.");
}
```

In this example, `list_chunks` is being used to query which files are available for the specified site and date, and
`download_chunk` is used to download the contents of the first file. The downloaded "chunk"/file will need to be
decompressed and decoded before the data can be inspected.

## Decompression

Raw chunk files are compressed with bzip2 and must be decompressed prior to decoding. Here is an example of 
decompressing a chunk file: 
```rust
let compressed_chunk = ...;

println!("Chunk data size (bytes): {}", compressed_chunk.data().len());
println!("Chunk data is compressed: {}", compressed_chunk.compressed());

let decompressed_chunk = decompress_chunk(&compressed_chunk)?;
println!("Decompressed chunk data size (bytes): {}", decompressed_chunk.data().len());
println!("Decompressed chunk data is compressed: {}", decompressed_chunk.compressed());
```

## Decoding

A decompressed chunk file consists of binary-encoded messages containing sweep data. Here is an example of decoding a 
chunk file:
```rust
let chunk = ...;
let decoded = decode_chunk(&chunk)?;
println!("Decoded chunk: {:?}", decoded);
```

Chunks contain a wide variety of data/message, but you may only be interested in a particular subset. To reduce the
volume of data that needs to be stored or processed, you can apply a filter when decoding a chunk. For example:
```rust
// TODO
```

The decoded chunk models are binary-serializable, so they can then be cached to disk for reuse later.
