# Radar Site Registry Maintenance

`nexrad-model/src/meta/registry.rs` contains the radar locations used by the
library. A missing or inaccurate entry can cause an application to choose a
farther radar, so the registry is checked against NOAA's published radar site
catalog.

## Source and Snapshot

The source snapshot is
`nexrad-model/data/operational-radar-sites.csv`. Its `last_verified` row is
the date on which the list was downloaded from the NOAA NWS GeoServer
`nws:radar_sites` layer. That catalog describes installed sites independently
of whether a site is temporarily unavailable, unlike lists inferred from
currently available Level II files.

The audit requests NOAA's CSV representation and reads and writes CSV with the
Rust standard library. Its HTTP client and command-line/runtime support reuse
dependencies already present in the workspace; the maintenance feature adds no
third-party dependencies.

The snapshot stores both names:

- `registry_name` is the concise city or installation name exposed by
  `nexrad-model`.
- `source_name` is NOAA's name and is retained so upstream name changes are
  detected.

Coordinates are compared with a 0.01 degree tolerance against the Rust
registry, which stores four decimal places. Successive NOAA snapshots use a
0.0001 degree tolerance and elevations use a one-meter tolerance.

The operational U.S. WSR-88D filter includes four-character `K` and `P` site
identifiers plus `TJUA`. It excludes `KBIX`, `KCRI`, `KLIX`, and `KOUN`, which
NOAA's geographic layer includes but which are not separate sites in the
156-site operational registry. This explicit policy makes a source or network
change fail visibly instead of silently changing the public registry.

## Automatic Test Behavior

The `xtask/tests/radar_site_registry.rs` integration test always compares the
checked-in snapshot with the Rust registry. If the snapshot is more than 31
days old, the test also downloads NOAA's current catalog and compares:

- added or removed operational identifiers;
- NOAA site names;
- latitude and longitude; and
- elevation.

A fresh snapshot keeps ordinary test runs offline. A stale snapshot triggers a
live read but never modifies tracked files. If NOAA has changed, the test fails
with a field-by-field report and writes the newly downloaded candidate to
`target/radar-sites-current.csv`. A source outage also fails the stale audit,
because an old registry must not be mistaken for a verified one.
Requests have a 10-second connection timeout and a 30-second total timeout.
Non-finite numbers and out-of-range coordinates are rejected before comparison
or snapshot updates.

The check command prints the number of sites compared and explicitly reports
whether the NOAA live check passed or was skipped because the snapshot is still
fresh.

To run the same checks manually:

```bash
# Use the age policy: fetch only when the snapshot is stale.
cargo run -p xtask -- check-radar-sites

# Fetch and compare even when the snapshot is fresh.
cargo run -p xtask -- check-radar-sites --force
```

## Updating the Snapshot

Run:

```bash
cargo run -p xtask -- update-radar-sites
```

This command downloads NOAA's catalog, rebuilds the tracked CSV snapshot, and
then checks it against the Rust registry. If NOAA added, removed, renamed, or
moved a site, the command leaves the new snapshot available for inspection and
fails with the registry changes still required.

Review both the NOAA source change and the generated diff before editing the
registry. Then rerun the update command, followed by:

```bash
cargo fmt --all -- --check
cargo clippy --all-features --workspace -- -D warnings
cargo test --all-features --workspace
```

Commit the registry and snapshot changes together. Even when NOAA data is
unchanged, maintainers may run the update command to advance `last_verified`
and keep normal test runs offline for the next 31 days.
