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
NOAA's geographic layer still lists but which are decommissioned or not part of
the operational network, so they are not in the 156-site registry. This
explicit policy makes a source or network change fail visibly instead of
silently changing the public registry.

Registry elevations are not compared with the snapshot. NOAA's `elevmeter`
values run about 30 meters above the registry's on average, so the snapshot
elevations are only used to detect changes in NOAA's own data.

## Checks

There are two checks, so ordinary development never depends on the network:

- **Offline check (default test suite).** `xtask/tests/radar_site_registry.rs`
  compares the checked-in snapshot with the Rust registry on every
  `cargo test --workspace`. It fails if either was edited without the other.
- **Live NOAA check (scheduled).** The `radar-site-audit` workflow runs weekly
  and on demand. It downloads NOAA's current catalog and compares it with the
  snapshot for:
  - added or removed operational identifiers;
  - NOAA site names;
  - latitude and longitude; and
  - elevation.

The live check never modifies tracked files. If NOAA has changed, it fails
with a field-by-field report, writes the newly downloaded candidate to
`target/radar-sites-current.csv`, uploads it as a workflow artifact, and opens
(or comments on) a GitHub issue. A source outage also fails the check, because
an unreachable source must not be mistaken for a verified registry. Requests
have a 10-second connection timeout and a 30-second total timeout. Non-finite
numbers and out-of-range coordinates are rejected before comparison or
snapshot updates. NOAA's layer can list a site more than once; identical
rows are collapsed, and rows that disagree are an error.

To run the same checks manually:

```bash
# Offline: compare the Rust registry with the checked-in snapshot.
cargo run -p xtask -- check-radar-sites

# Live: also compare the snapshot with NOAA's current catalog.
cargo run -p xtask -- check-radar-sites --live
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
unchanged, maintainers may run the update command to advance `last_verified`.
