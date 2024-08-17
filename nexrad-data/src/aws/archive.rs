//!
//! TODO: explain how the archive bucket is structured/works
//!

use crate::archive::{File, Identifier};
use crate::aws::s3::{download_object, list_objects};
use crate::result::{Error, Result};
use chrono::NaiveDate;

const ARCHIVE_BUCKET: &str = "noaa-nexrad-level2";

/// List data files for the specified site and date. This effectively returns an index of data files
/// which can then be individually downloaded.
pub async fn list_files(site: &str, date: &NaiveDate) -> Result<Vec<Identifier>> {
    let prefix = format!("{}/{}", date.format("%Y/%m/%d"), site);
    let list_result = list_objects(ARCHIVE_BUCKET, &prefix, None).await?;
    if list_result.truncated {
        return Err(Error::TruncatedListObjectsResponse);
    }

    let metas = list_result
        .objects
        .iter()
        .map(|object| {
            let key_parts = object.key.split('/');
            let name = key_parts.skip(4).collect::<String>();

            Identifier::new(name)
        })
        .collect();

    Ok(metas)
}

/// Download a data file specified by its metadata. Returns the downloaded file's encoded contents
/// which may then need to be decompressed and decoded.
pub async fn download_file(identifier: Identifier) -> Result<File> {
    let date = identifier
        .date_time()
        .ok_or_else(|| Error::DateTimeError(identifier.name().to_string()))?;

    let site = identifier
        .site()
        .ok_or_else(|| Error::InvalidSiteIdentifier(identifier.name().to_string()))?;

    let key = format!("{}/{}/{}", date.format("%Y/%m/%d"), site, identifier.name());
    let data = download_object(ARCHIVE_BUCKET, &key).await?;

    Ok(File::new_with_identifier(identifier, data))
}
