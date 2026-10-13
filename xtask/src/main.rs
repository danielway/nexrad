use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "Repository maintenance commands for nexrad")]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check the Rust registry against the checked-in radar site snapshot.
    CheckRadarSites {
        /// Also compare the snapshot with NOAA's current catalog (requires network access).
        #[arg(long)]
        live: bool,
    },
    /// Download NOAA data and rebuild the checked-in radar site snapshot.
    UpdateRadarSites,
    /// Regenerate the public API baselines under api/ (requires cargo-public-api and nightly).
    PublicApi {
        /// Fail with a diff instead of rewriting baselines that are out of date.
        #[arg(long)]
        check: bool,
    },
}

#[tokio::main]
async fn main() -> xtask::Result<()> {
    match Arguments::parse().command {
        Command::CheckRadarSites { live: false } => {
            let snapshot = xtask::check_registry_offline()?;
            println!(
                "OK: Rust registry matches the checked-in snapshot ({} sites). \
                 Pass --live to also compare with NOAA.",
                snapshot.sites.len()
            );
            Ok(())
        }
        Command::CheckRadarSites { live: true } => xtask::check_against_noaa().await,
        Command::UpdateRadarSites => xtask::update_radar_sites().await,
        Command::PublicApi { check: false } => xtask::public_api::update(),
        Command::PublicApi { check: true } => xtask::public_api::check(),
    }
}
