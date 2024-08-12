use nexrad_data::result::Result;

#[cfg(feature = "aws")]
use nexrad_data::aws::realtime::{get_latest_volume, list_chunks};

#[cfg(not(feature = "aws"))]
fn main() {
    println!("This example requires the \"aws\" feature to be enabled.");
}

#[cfg(feature = "aws")]
#[tokio::main]
async fn main() -> Result<()> {
    // for chunk in 0..=999 {
    //     let files = list_chunks("KDMX", Some(chunk), 1).await?;
    //     if files.is_empty() {
    //         println!("Chunk {:03}: No files found.", chunk);
    //     } else {
    //         println!("Chunk {:03}: {}", chunk, files.first().unwrap().1);
    //     }
    // }

    let latest = get_latest_volume("KDMX").await?;
    println!("Most recent volume: {:?}", latest);

    let chunks = list_chunks("KDMX", latest.unwrap(), 1000).await?;
    println!("Found {} chunks.", chunks.len());

    for (name, date) in chunks {
        println!("{}: {}", name, date);
    }

    Ok(())
}
