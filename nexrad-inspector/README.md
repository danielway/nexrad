# nexrad-inspector

Interactive TUI for inspecting NEXRAD Archive II volume files.

## Features

- Browse and open local NEXRAD Archive II files
- Download files directly from AWS NEXRAD archive
- Decompress and inspect individual LDM records
- Parse and view radar messages with hex and parsed views
- Save records and messages to files for further analysis
- Vim-style keyboard navigation

## Installation

```bash
cargo install nexrad-inspector
```

## Usage

Launch the inspector interactively:

```bash
nexrad-inspector
```

Or open a specific file directly:

```bash
nexrad-inspector path/to/volume/file
```

## Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `Enter` | Select / drill down |
| `Esc` / `Backspace` | Go back |
| `Tab` | Toggle hex/parsed view |
| `d` | Decompress selected record |
| `D` | Decompress all records |
| `s` | Save current record/message |
| `?` | Toggle help |
| `q` | Quit |

## Views

The inspector provides a hierarchical view of volume files:

1. **File View** - List of LDM records in the volume file
2. **Record View** - List of messages within a decompressed record
3. **Message View** - Hex dump and parsed view of message contents

## AWS Browser

The AWS browser allows you to download files directly from NOAA's NEXRAD archive on AWS:

1. Enter a radar site identifier (e.g., `KDMX`)
2. Enter a date in `YYYY-MM-DD` format
3. Select a file from the list to download and inspect

Downloaded files are cached in a `./downloads` directory for faster subsequent access.
