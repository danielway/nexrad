//! Public API baselines for the published library crates.
//!
//! Each library crate's public API, as listed by `cargo public-api`, is committed under `api/`
//! so that API changes show up in review. [`update`] regenerates the baselines and [`check`]
//! fails if any of them is stale.

use crate::{error, with_context, workspace_root, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Published library crates that have a public API baseline. `nexrad-inspector` and `xtask`
/// are not published and are deliberately absent.
const LIBRARY_CRATES: &[&str] = &[
    "nexrad-model",
    "nexrad-decode",
    "nexrad-data",
    "nexrad-process",
    "nexrad-render",
    "nexrad",
];

/// Nightly toolchain used to produce rustdoc JSON. It is pinned because both the rustdoc JSON
/// format and the standard library's auto traits change between nightlies, which would make
/// baselines generated on different days disagree. Keep in sync with the `public-api` job in
/// `.github/workflows/ci.yml`.
const NIGHTLY_TOOLCHAIN: &str = "nightly-2026-10-11";

/// Directory holding the committed baselines.
fn baseline_dir() -> PathBuf {
    workspace_root().join("api")
}

/// Path of the committed baseline for `crate_name`.
fn baseline_path(crate_name: &str) -> PathBuf {
    baseline_dir().join(format!("{crate_name}.txt"))
}

/// Lists the public API of `crate_name` with all features enabled.
fn generate(crate_name: &str) -> Result<String> {
    let manifest = workspace_root().join(crate_name).join("Cargo.toml");
    eprintln!("Listing public API of {crate_name} with {NIGHTLY_TOOLCHAIN}");

    let output = with_context(
        Command::new("cargo")
            .env("RUSTUP_TOOLCHAIN", NIGHTLY_TOOLCHAIN)
            .arg("public-api")
            .arg("--manifest-path")
            .arg(&manifest)
            .args(["--all-features", "--simplified", "--color", "never"])
            .output(),
        "failed to run `cargo public-api` (install it with `cargo install cargo-public-api --locked`)",
    )?;

    if !output.status.success() {
        return Err(error(format!(
            "`cargo public-api` failed for {crate_name}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    with_context(
        String::from_utf8(output.stdout),
        format!("`cargo public-api` printed non-UTF-8 output for {crate_name}"),
    )
}

/// Regenerates every baseline under `api/`.
pub fn update() -> Result<()> {
    with_context(fs::create_dir_all(baseline_dir()), "failed to create api/")?;
    for crate_name in LIBRARY_CRATES {
        let api = generate(crate_name)?;
        let path = baseline_path(crate_name);
        with_context(
            fs::write(&path, api),
            format!("failed to write {}", path.display()),
        )?;
        println!("Wrote {}", path.display());
    }
    Ok(())
}

/// Compares every baseline under `api/` with the current public API, printing a diff for each
/// stale baseline. Fails if any baseline is missing or stale.
pub fn check() -> Result<()> {
    let scratch_dir = workspace_root().join("target/public-api");
    with_context(
        fs::create_dir_all(&scratch_dir),
        format!("failed to create {}", scratch_dir.display()),
    )?;

    let mut stale = Vec::new();
    for crate_name in LIBRARY_CRATES {
        let current = generate(crate_name)?;
        let baseline_path = baseline_path(crate_name);
        let baseline = fs::read_to_string(&baseline_path).unwrap_or_default();
        if baseline == current {
            continue;
        }

        let current_path = scratch_dir.join(format!("{crate_name}.txt"));
        with_context(
            fs::write(&current_path, &current),
            format!("failed to write {}", current_path.display()),
        )?;
        print_diff(&baseline_path, &current_path)?;
        stale.push(*crate_name);
    }

    if stale.is_empty() {
        println!(
            "OK: public API baselines are up to date ({} crates).",
            LIBRARY_CRATES.len()
        );
        Ok(())
    } else {
        Err(error(format!(
            "public API baselines are stale for: {}. \
             Run `cargo run -p xtask -- public-api` and commit the changes under api/.",
            stale.join(", ")
        )))
    }
}

/// Prints a unified diff from the committed baseline to the freshly generated API, using paths
/// relative to the workspace root.
fn print_diff(baseline: &Path, current: &Path) -> Result<()> {
    let root = workspace_root();
    let relative = |path: &Path| path.strip_prefix(&root).unwrap_or(path).to_path_buf();
    with_context(
        Command::new("git")
            .current_dir(&root)
            .args(["--no-pager", "diff", "--no-index", "--"])
            .arg(relative(baseline))
            .arg(relative(current))
            .status(),
        "failed to run `git diff`",
    )?;
    Ok(())
}
