# NEXRAD

[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

A suite of tools for working with NEXRAD WSR-88D weather radar data.

## `nexrad`

[![Crate](https://img.shields.io/crates/v/nexrad.svg)](https://crates.io/crates/nexrad)
[![Docs.rs](https://docs.rs/nexrad/badge.svg)](https://docs.rs/nexrad)

Ergonomic APIs for accessing, decoding, and processing NEXRAD weather radar data.

## `nexrad-model`

[![Crate](https://img.shields.io/crates/v/nexrad-model.svg)](https://crates.io/crates/nexrad-model)
[![Docs.rs](https://docs.rs/nexrad-model/badge.svg)](https://docs.rs/nexrad-model)

A common model for representing NEXRAD weather radar data. Provides an ergonomic API which is documented for an audience
who is not necessarily familiar with the NOAA Archive II format.

## `nexrad-decode`

[![Crate](https://img.shields.io/crates/v/nexrad-decode.svg)](https://crates.io/crates/nexrad-decode)
[![Docs.rs](https://docs.rs/nexrad-decode/badge.svg)](https://docs.rs/nexrad-decode)

Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are in accordance with
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002W".

## `nexrad-data`

[![Crate](https://img.shields.io/crates/v/nexrad-data.svg)](https://crates.io/crates/nexrad-data)
[![Docs.rs](https://docs.rs/nexrad-data/badge.svg)](https://docs.rs/nexrad-data)

Download and processing functions for NEXRAD weather radar data.

## `nexrad-render`

[![Crate](https://img.shields.io/crates/v/nexrad-render.svg)](https://crates.io/crates/nexrad-render)
[![Docs.rs](https://docs.rs/nexrad-render/badge.svg)](https://docs.rs/nexrad-render)

Functions for rendering NEXRAD weather radar data into visual images.

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
