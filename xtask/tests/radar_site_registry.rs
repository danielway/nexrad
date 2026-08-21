#[test]
fn checked_in_radar_sites_match_registry_and_check_noaa_when_stale() -> xtask::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(xtask::check_radar_sites(false))
}
