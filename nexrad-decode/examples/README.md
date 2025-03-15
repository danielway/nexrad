# NEXRAD Decode Examples

This directory contains examples demonstrating how to use the `nexrad-decode` library for working with NEXRAD weather radar data.

## Available Examples

### Elevation Angles CSV Generator

The `elevation_angles.rs` example demonstrates how to extract elevation angle data from a NEXRAD Archive II file and generate a CSV file. 
The CSV has:
- Columns representing elevation numbers (elev_1, elev_2, etc.)
- Rows representing azimuth numbers
- Values at the intersection representing the elevation angle for that elevation/azimuth number combination

#### Usage

```bash
# Run with default output filename (elevation_angles.csv)
cargo run --example elevation_angles -- /path/to/archive_file.ar2

# Specify a custom output filename
cargo run --example elevation_angles -- /path/to/archive_file.ar2 --output-path custom_output.csv
```

#### Example Output Format

```
azimuth_num,elev_1,elev_2,elev_3
1,0.52,1.45,2.41
2,0.52,1.46,2.40
...
```

Note: Empty cells in the CSV indicate that no data was available for that particular elevation/azimuth number combination. 