use crate::aws::archive::identifier::Identifier;
use crate::aws::archive::ARCHIVE_BUCKET;
use crate::aws::s3::list_objects;
use crate::result::AwsError::TruncatedListObjectsResponse;
use crate::result::Error::Aws;
use chrono::NaiveDate;

/// A volume file in an archive listing, with its stored size.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ListedFile {
    identifier: Identifier,
    size: u64,
}

impl ListedFile {
    /// The file's identifier, which can be passed to [`download_file`](super::download_file).
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// The file's compressed size in bytes, as reported by the archive bucket.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Consumes the listing entry, returning its identifier.
    pub fn into_identifier(self) -> Identifier {
        self.identifier
    }
}

/// List data files for the specified site and date. This effectively returns an index of data files
/// which can then be individually downloaded.
pub async fn list_files(site: &str, date: NaiveDate) -> crate::result::Result<Vec<Identifier>> {
    let files = list_files_with_sizes(site, date).await?;
    Ok(files.into_iter().map(ListedFile::into_identifier).collect())
}

/// List data files for the specified site and date along with each file's size in bytes. Sizes come
/// from the same listing request as [`list_files`], so this costs nothing extra.
pub async fn list_files_with_sizes(
    site: &str,
    date: NaiveDate,
) -> crate::result::Result<Vec<ListedFile>> {
    let prefix = format!("{}/{}", date.format("%Y/%m/%d"), site);
    let list_result = list_objects(ARCHIVE_BUCKET, &prefix, None).await?;
    if list_result.truncated {
        return Err(Aws(TruncatedListObjectsResponse));
    }

    let files = list_result
        .objects
        .into_iter()
        .map(|object| {
            let name = object.key.split('/').skip(4).collect::<String>();

            ListedFile {
                identifier: Identifier::new(name),
                size: object.size,
            }
        })
        .collect();

    Ok(files)
}
