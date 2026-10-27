# Release Process

This repository uses GitHub Actions to manage the release process for individual crates. Each crate is versioned independently using the format `crate-name@x.y.z`.

## Pre-Release Checklist

Before running the release workflow for a crate, confirm on `main`:

- [ ] The crate's `CHANGELOG.md` is final: move the entries under `## [Unreleased]` into a new
      `## [x.y.z] - YYYY-MM-DD` section (dated with the release day), leave an empty `## [Unreleased]` above it,
      and check that every user-visible change since the last release is listed.
- [ ] `cargo run -p xtask -- public-api --check` passes, i.e. the `Public API baselines` CI job is green and
      `api/<crate>.txt` describes exactly what will be published.
- [ ] The `Semver checks` CI job output has been reviewed. The version type you choose must match what it reports:
      any breaking change needs a major bump.
- [ ] `cargo publish --dry-run -p <crate>` succeeds for the crate. When releasing several crates, dry-run them in
      dependency order (see [Release Order](#release-order)); a dependent's dry run only succeeds once the version it
      requires is on crates.io.
- [ ] The `MSRV check` CI job is green.

### Rules that apply between releases

- **API changes update the baselines in the same PR.** Any PR that changes a crate's public API runs
  `cargo run -p xtask -- public-api` and commits the regenerated `api/<crate>.txt`, and adds a CHANGELOG entry under
  `## [Unreleased]`.
- **MSRV bumps are minor releases.** The MSRV (`rust-version` in the workspace `Cargo.toml`) is only raised in a minor
  release, never in a patch release, and the raise is noted under "Changed" in the CHANGELOG of every affected crate.
- **CHANGELOGs follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).** Each library crate has a
  `CHANGELOG.md` with an `## [Unreleased]` section and `### Added`, `### Changed`, `### Deprecated`, `### Removed`
  and `### Fixed` subsections as needed.

## README Images

Each crate README links its image by absolute URL at the crate's release tag, for example
`https://raw.githubusercontent.com/danielway/nexrad/nexrad-render@1.0.0/docs/images/products.png`. crates.io needs
absolute URLs, and the tag keeps a published README showing the image it was released with even after `main` changes.
The link 404s until that tag exists.

To change an image, regenerate it (`cargo run -p xtask -- readme-images`), and in the same PR point the README links
of the crates that show it at the tag of their next release. Each crate's tag must contain the image, so merge that PR
before running the version bump. A release that doesn't change an image can keep linking the older tag.

## Release Steps

1. **Initiate Version Bump**
   - Go to the "Actions" tab in GitHub
   - Select the "Release Bump" workflow
   - Click "Run workflow" and select:
     - The crate to release
     - Version type:
       - `major` for breaking changes (x.0.0)
       - `minor` for new features (0.x.0)
       - `patch` for bug fixes (0.0.x)
       - `rc` for release candidates (0.0.x-rc.1)
       - `custom` for arbitrary versions (e.g., 1.0.0-rc.1)
     - If using `custom`, enter the exact version string in the "Custom version" field

2. **Review Draft Release**
   - The workflow will:
     - Create a new version commit
     - Tag it with `crate-name@x.y.z`
     - Create a draft GitHub release
   - Go to the "Releases" page
   - Find the draft release
   - Edit the release notes: paste the crate's CHANGELOG section for this version
   - Ensure the tag name follows the format: `crate-name@x.y.z`

3. **Publish Release**
   - Click "Publish release" in the GitHub UI
   - This will trigger the "Publish Crate" workflow automatically
   - The workflow will:
     - Run CI checks
     - Publish the crate to crates.io

## Available Crates

The following crates can be released:
- `nexrad`
- `nexrad-data`
- `nexrad-decode`
- `nexrad-model`
- `nexrad-process`
- `nexrad-render`

`nexrad-inspector` and `xtask` are `publish = false` and are never released.

## Crate Dependencies

The crates have interdependencies that affect release order:

```
              nexrad-model (no internal deps)
                   │
     ┌─────────────┼──────────────┐
     ▼             ▼              ▼
nexrad-decode  nexrad-process  nexrad-render
     │             │              │
     ▼             │              │
nexrad-data        │              │
     │             │              │
     └─────────────┼──────────────┘
                   ▼
            nexrad (facade, depends on all five)
```

### Release Order

When releasing multiple crates, follow this order:

1. **nexrad-model** - foundation crate, no internal dependencies
2. **nexrad-decode**, then **nexrad-process**, then **nexrad-render** - all depend on nexrad-model; publish each before
   bumping the next
3. **nexrad-data** - depends on nexrad-decode and nexrad-model
4. **nexrad** - facade crate, depends on all others

### The 1.0.0 Release

All six library crates release 1.0.0 together. On the day of release:

- [ ] In one PR to `main`, finalize every crate's CHANGELOG as the first [pre-release checklist](#pre-release-checklist)
      item describes, with `1.0.0` as the version.
- [ ] Release the crates one at a time in [release order](#release-order). For each crate, run the rest of the
      pre-release checklist and the [release steps](#release-steps), choosing the `custom` version type and `1.0.0`
      (`major` would produce `2.0.0`). Check that the bump commit makes the changes
      [Automatic Dependency Updates](#automatic-dependency-updates) describes; no requirement needs a manual edit.
      Wait for the Publish Crate workflow to finish before bumping the next crate.
- [ ] After `nexrad` 1.0.0 is published, remove `continue-on-error: true` and the comment above it from the
      `Semver checks` job in `.github/workflows/ci.yml`, so semver violations against the 1.0.0 releases fail CI.
- [ ] Confirm `cargo run -p xtask -- public-api --check` passes on `main` after the last bump commit. The baselines
      under `api/` and `api/README.md` record no versions and need no change.

### Automatic Dependency Updates

The Release Bump workflow uses `cargo release`, which sets the crate's version, updates the version requirement of
every workspace dependency and dev-dependency on the released crate that has both a `version` and a `path` (for
example `nexrad-render`'s dev-dependency on `nexrad-process` and the facade's dependencies on all five crates), updates
`Cargo.lock`, and commits those changes with the version bump. Path-only dependencies (`nexrad-inspector`, `xtask`)
are not touched.

You still need to release crates in dependency order because each crate must be published to crates.io before its
dependents can be released.

## Troubleshooting

- If the publish workflow fails, check:
  - The release tag format (must be `crate-name@x.y.z`)
  - That the version in `Cargo.toml` matches the tag
  - The crates.io token is properly set in repository secrets
  - All CI checks passed

## Notes

- Release candidates are marked as pre-releases on GitHub
- The publish workflow will only run when a release is published (not drafted)
