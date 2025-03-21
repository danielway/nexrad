use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Download the latest NEXRAD archive and convert to NetCDF")]
struct Args {
    /// The NEXRAD site to download
    #[arg(short, long, default_value = "KDMX")]
    site: String,

    /// Path to the output NetCDF file
    #[arg(short, long, default_value = "output.nc")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    println!("This example will download the latest NEXRAD archive for site {} and convert it to NetCDF at: {:?}", args.site, args.output);

    // TODO: Implement example functionality

    Ok(())
}
