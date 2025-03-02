# NEXRAD

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

Provides data structures decoding, and data access functionality for NEXRAD Archive Level II data.

The [NEXRAD system](https://www.ncei.noaa.gov/products/radar/next-generation-weather-radar) provides
high-resolution weather radar data across North America and other regions. Data from these radars is
processed and available at Level II which contains base data and Level III which contains a number
of derived "products". Level II is the highest resolution data available including base data
(reflectivity, mean radial velocity, and spectrum width) and dual polarization variables
(differential reflectivity, correlation coefficient, and differential phase).

Level II data is available through the Archive II interface described by NOAA's ICD 2620010H.
Section 7 of the ICD specifies the format for this API. This data format is distributed through
Unidata Local Data Manager (LDM) software. The data is organized into "volumes" (a file with binary
data) which contain a number of compressed "LDR records", each of which contain "messages" that
correspond to radials/rays from the radar with corresponding data and parameters.
