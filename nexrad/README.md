# NEXRAD

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Decode and download functions for NEXRAD WSR-88D weather radar data.

## Summary

This library provides data structures and decoding functionality for NEXRAD Archive Level II data. Optionally, it also
provides decompression and download functionality for near real-time data uploaded to AWS by NOAA.

The [NEXRAD system](https://www.ncei.noaa.gov/products/radar/next-generation-weather-radar) provides high-resolution
weather radar data across North America and other regions. Data from these radars is processed and available at Level II
which contains base data and Level III which contains a number of derived "products". Level II is the highest resolution
data available including base data (reflectivity, mean radial velocity, and spectrum width) and dual polarization
variables (differential reflectivity, correlation coefficient, and differential phase).

Level II data is available through the Archive II interface described by NOAA's ICD 2620010H. Section 7 of the ICD
specifies the format for this API. This data format is distributed through Unidata Local Data Manager (LDM) software.
The data is organized into "volumes" (a file with binary data) which contain a number of compressed "LDR records", each
of which contain "messages" that correspond to radials/rays from the radar with corresponding data and parameters.

<img src="examples/render_kdmx_030522_1730.png" width="400" alt="An EF4 tornado near Des Moines, IA on March 5, 2022 rendered using this library's 'render' example." />

_An EF4 tornado near Des Moines, IA on March 5, 2022 rendered using this library's "render" example._

## Downloading

The `download` feature may be enabled to download NEXRAD Archive II data from AWS. For more information on this data
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
    let downloaded_file = download_file(meta).await?;
    
    println!("Data file size (bytes): {}", downloaded_file.len());
    println!("Data file is compressed: {}", is_compressed(downloaded_file));
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
let decompressed_file = ...;

let decoded = decode_file(&decompressed_file)?;
println!("Decoded file with {} elevations.", decoded.elevation_scans().len());
```

## Rendering

A downloaded file can be rendered to an image using the `render` example. Here is an example usage and the result:
```
cargo run --example render KDMX20220305_233003_V06
```

## Acknowledgements

I consulted the following resources when developing this library:

NOAA NCEI, NEXRAD System and Product Description with Access Information:
https://www.ncei.noaa.gov/products/radar/next-generation-weather-radar

NOAA NWS, Radar Operations Center, NEXRAD WSR-88D Level II Data Information:
https://www.roc.noaa.gov/wsr88d/level_ii/level2info.aspx

NOAA NWS, Radar Operations Center, NEXRAD WSR-88D Interface Control Documents:
https://www.roc.noaa.gov/wsr88d/BuildInfo/Files.aspx

NASA TRMM, Radar Software Library:
https://trmm-fc.gsfc.nasa.gov/trmm_gv/software/rsl/

Brian Wigginton, a Go implementation of NEXRAD Level II decoding:
https://github.com/bwiggs/go-nexrad
