# NEXRAD Decode

[![Crates.io](https://img.shields.io/crates/v/nexrad-decode)](https://crates.io/crates/nexrad-decode)
[![Docs.rs](https://docs.rs/nexrad-decode/badge.svg)](https://docs.rs/nexrad-decode)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)
[![Rust CD](https://github.com/danielway/nexrad/actions/workflows/cd.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/cd.yml)

Decoding functions and models for NEXRAD weather radar data. Decoder and struct definitions are in accordance with
NOAA's WSR-88D Interface Control Document for the RDA/RPG "ICD 2620002W".

## Features

- `nexrad-model`: Provides mappings to a common model for representing NEXRAD radar data.
- `uom`: Use the `uom` crate for type-safe units of measure.
