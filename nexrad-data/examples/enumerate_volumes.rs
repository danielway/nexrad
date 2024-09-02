#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{list_chunks_in_volume, VolumeIndex};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> nexrad_data::result::Result<()> {
    for volume in 0..998 {
        let chunks = list_chunks_in_volume("KDMX", VolumeIndex::new(volume + 1), 1).await?;
        if chunks.len() == 0 {
            println!("Volume {:03} empty.", volume + 1);
        } else {
            println!("Volume {:03} time: {:?}", volume + 1, chunks[0].date_time());
        }
    }
    Ok(())
}