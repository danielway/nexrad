# Release Process

This repository uses GitHub Actions to manage the release process for individual crates. Each crate is versioned independently using the format `crate-name@x.y.z`.

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
   - Edit the release notes to add the changelog
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
- `nexrad-render`

## Crate Dependencies

The crates have interdependencies that affect release order:

```
nexrad-model (no internal deps)
       │
   ┌───┴───┐
   ▼       ▼
nexrad-decode   nexrad-render
   │
   ▼
nexrad-data

       │ (all four)
       ▼
     nexrad (facade)
```

### Release Order

When releasing multiple crates, follow this order:

1. **nexrad-model** - foundation crate, no internal dependencies
2. **nexrad-decode** and **nexrad-render** - both depend only on nexrad-model (can be released in parallel)
3. **nexrad-data** - depends on nexrad-decode and nexrad-model
4. **nexrad** - facade crate, depends on all others

### Automatic Dependency Updates

The Release Bump workflow uses `cargo release`, which automatically updates version requirements in all dependent crates within the workspace and commits those changes. For example, releasing `nexrad-model@1.0.0-rc.1` will also update the `nexrad-model` version in `nexrad-decode/Cargo.toml`, `nexrad-render/Cargo.toml`, etc.

You still need to release crates in dependency order because each crate must be published to crates.io before its dependents can be released

## Troubleshooting

- If the publish workflow fails, check:
  - The release tag format (must be `crate-name@x.y.z`)
  - That the version in `Cargo.toml` matches the tag
  - The crates.io token is properly set in repository secrets
  - All CI checks passed

## Notes

- Each crate is versioned independently
- Release tags must follow the format `crate-name@x.y.z`
- Release candidates are marked as pre-releases on GitHub
- The publish workflow will only run when a release is published (not drafted)

