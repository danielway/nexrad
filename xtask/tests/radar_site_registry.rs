#[test]
fn checked_in_radar_sites_match_registry() -> xtask::Result<()> {
    let snapshot = xtask::check_registry_offline()?;
    println!(
        "Rust registry matches the checked-in snapshot ({} sites).",
        snapshot.sites.len()
    );
    Ok(())
}

/// Compares the checked-in snapshot with NOAA's live catalog. Run by the scheduled
/// `radar-site-audit` workflow with `cargo test -p xtask -- --ignored`.
#[test]
#[ignore = "requires network access to NOAA"]
fn checked_in_radar_sites_match_noaa() -> xtask::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(xtask::check_against_noaa())
}
