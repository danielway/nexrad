use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "Repository maintenance commands for nexrad")]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check the Rust registry and, when stale, compare it with NOAA.
    CheckRadarSites {
        /// Download NOAA data even when the checked-in snapshot is fresh.
        #[arg(long)]
        force: bool,
    },
    /// Download NOAA data and rebuild the checked-in radar site snapshot.
    UpdateRadarSites,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Arguments::parse().command {
        Command::CheckRadarSites { force } => xtask::check_radar_sites(force).await,
        Command::UpdateRadarSites => xtask::update_radar_sites().await,
    }
}
