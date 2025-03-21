# NEXRAD NetCDF

[![Crates.io](https://img.shields.io/crates/v/nexrad-netcdf)](https://crates.io/crates/nexrad-netcdf)
[![Docs.rs](https://docs.rs/nexrad-netcdf/badge.svg)](https://docs.rs/nexrad-netcdf)
[![Rust CI](https://github.com/danielway/nexrad/actions/workflows/ci.yml/badge.svg)](https://github.com/danielway/nexrad/actions/workflows/ci.yml)

NetCDF file format support for NEXRAD data.

## Overview

The `nexrad-netcdf` crate provides functionality for converting NEXRAD weather radar data to NetCDF
(Network Common Data Form) formats such as `CfRadial2`. This enables interoperability with a wide
range of scientific software and visualization tools that support NetCDF.

## Features

- Convert NEXRAD volume scans to NetCDF format
- CF-compliant metadata for meteorological data
