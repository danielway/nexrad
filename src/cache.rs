//!
//! Provides functions for caching fetched NEXRAD data and the result of expensive operations like
//! decompressing.
//!

use std::collections::HashSet;
use std::fs::{create_dir_all, File, read_dir};
use std::io::Write;
use std::path::Path;

use chrono::NaiveDate;

use crate::chunk::ChunkMeta;
use crate::decompress::decompress_chunk;
use crate::fetch::{fetch_chunk, list_chunks};
use crate::result::Result;

/// Specifies parameters for chunk download/decompression caching.
pub struct CacheConfig {
    directory: String,
    decompress: bool,
    fetch_handler: Option<fn(&ChunkMeta)>,
    decompress_handler: Option<fn(&ChunkMeta)>,
}

impl CacheConfig {
    /// Creates a new [CacheConfig] with the specified cache directory.
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
            decompress: false,
            fetch_handler: None,
            decompress_handler: None,
        }
    }

    /// If true, will also decompress the chunks after fetching them.
    pub fn decompress(mut self, decompress: bool) -> Self {
        self.decompress = decompress;
        self
    }

    /// Sets a handler to be called as chunks are fetched.
    pub fn fetch_handler(mut self, handler: fn(&ChunkMeta)) -> Self {
        self.fetch_handler = Some(handler);
        self
    }

    /// Sets a handler to be called as chunks are decompressed.
    pub fn decompress_handler(mut self, handler: fn(&ChunkMeta)) -> Self {
        self.decompress_handler = Some(handler);
        self
    }
}

/// Fetches and caches all available chunks for the specified date. If previously cached, this will
/// not re-fetch the data. The [config] may be used to specify this function's behavior.
///
/// Chunks are cached in the [CacheConfig.directory()] in the following structure:
///     {CacheConfig.directory()}/{year}/{month}/{day}/{site}/{chunk_id}
pub async fn update_cache(
    site: &str,
    date: &NaiveDate,
    config: CacheConfig,
) -> Result<Vec<ChunkMeta>> {
    // Ensure the cache directory for this particular date/site exists
    let directory = format!("{}/{}/{}", config.directory, date.format("%Y/%m/%d"), site);
    if !Path::new(&directory).exists() {
        create_dir_all(&directory)?;
    }

    // Load any previously-cached chunks for this date/site
    let mut cached_chunk_metas = HashSet::new();
    for entry in read_dir(&directory)? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            continue;
        }

        cached_chunk_metas.insert(ChunkMeta::new(
            site.to_string(),
            *date,
            entry.file_name().to_str().unwrap().to_string(),
        ));
    }

    // List all available chunks, and download and save any that are not cached
    let all_chunk_metas = list_chunks(site, date).await?;
    for meta in &all_chunk_metas {
        if !cached_chunk_metas.contains(&meta) {
            if let Some(handler) = config.fetch_handler {
                handler(&meta);
            }

            let chunk = fetch_chunk(&meta).await?;

            let mut file = File::create(format!("{}/{}", directory, meta.identifier()))?;
            file.write_all(&chunk.data())?;

            // Optionally decompress the chunk and save it to disk too
            if config.decompress && chunk.compressed() {
                if let Some(handler) = config.decompress_handler {
                    handler(&meta);
                }

                let decompressed_chunk = decompress_chunk(&chunk)?;

                let mut file = File::create(format!("{}/{}.decompressed", directory, meta.identifier()))?;
                file.write_all(&decompressed_chunk.data())?;
            }
        }
    }

    Ok(all_chunk_metas)
}