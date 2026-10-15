use crate::aws::archive::identifier::Identifier;
use crate::aws::archive::ARCHIVE_BUCKET;
use crate::aws::s3::list_objects;
use crate::result::AwsError::TruncatedListObjectsResponse;
use crate::result::Error::Aws;
use chrono::NaiveDate;
use std::cmp::Ordering;

/// A volume file in an archive listing, with its stored size.
///
/// Ordering follows the identifier (chronological within a site), with the size breaking ties so
/// that ordering agrees with equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl PartialOrd for ListedFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ListedFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identifier
            .cmp(&other.identifier)
            .then(self.size.cmp(&other.size))
    }
}

/// Lists the volume files for the specified site and UTC date. This effectively returns an index
/// of volume files which can then be individually downloaded.
///
/// Only volume files are returned: metadata objects ([`Identifier::is_metadata`]) and objects whose
/// names do not follow the archive grammar are skipped rather than reported as errors.
///
/// # Errors
///
/// Returns [`AwsError::S3ListObjects`](crate::result::AwsError::S3ListObjects) if the listing
/// request fails,
/// [`AwsError::S3ListObjectsDecoding`](crate::result::AwsError::S3ListObjectsDecoding) if its
/// response cannot be parsed, and
/// [`AwsError::TruncatedListObjectsResponse`](crate::result::AwsError::TruncatedListObjectsResponse)
/// if the listing is truncated. A date without data is not an error; it yields an empty list.
pub async fn list_files(site: &str, date: NaiveDate) -> crate::result::Result<Vec<Identifier>> {
    let files = list_files_with_sizes(site, date).await?;
    Ok(files.into_iter().map(ListedFile::into_identifier).collect())
}

/// Lists the volume files for the specified site and UTC date along with each file's size in
/// bytes. Sizes come from the same listing request as [`list_files`], so this costs nothing extra.
///
/// Only volume files are returned: metadata objects ([`Identifier::is_metadata`]) and objects whose
/// names do not follow the archive grammar are skipped rather than reported as errors.
///
/// # Errors
///
/// Returns the same errors as [`list_files`].
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
        .filter_map(|object| volume_file(&object.key, object.size))
        .collect();

    Ok(files)
}

/// Maps an archive object key (`YYYY/MM/DD/SITE/NAME`) to a listing entry, or `None` if the
/// object is not a volume file.
fn volume_file(key: &str, size: u64) -> Option<ListedFile> {
    let name = key.rsplit('/').next()?;
    let identifier = Identifier::parse(name).ok()?;
    (!identifier.is_metadata()).then_some(ListedFile { identifier, size })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_keeps_only_volume_files() {
        let keys = [
            "2022/03/05/KDMX/KDMX20220305_232324_V06",
            "2022/03/05/KDMX/KDMX20220305_232324_V06_MDM",
            "2022/03/05/KDMX/NWS_NEXRAD_NXL2DP_KDMX_20220305.tar",
            "2010/05/01/KTLX/KTLX20100501_000444_V03.gz",
        ];
        let names: Vec<_> = keys
            .iter()
            .filter_map(|key| volume_file(key, 1))
            .map(|file| file.identifier().name().to_string())
            .collect();
        assert_eq!(
            names,
            ["KDMX20220305_232324_V06", "KTLX20100501_000444_V03.gz"]
        );
    }

    #[test]
    fn listed_files_order_by_identifier_then_size() {
        let file = |name: &str, size| volume_file(name, size).unwrap();
        let mut files = vec![
            file("KDMX20220305_232324_V06", 5),
            file("KDMX20220305_120000_V06", 9),
            file("KDMX20220305_232324_V06", 3),
        ];
        files.sort();
        let order: Vec<_> = files
            .iter()
            .map(|file| (file.identifier().name(), file.size()))
            .collect();
        assert_eq!(
            order,
            [
                ("KDMX20220305_120000_V06", 9),
                ("KDMX20220305_232324_V06", 3),
                ("KDMX20220305_232324_V06", 5),
            ]
        );
        assert!(format!("{:?}", files[0]).contains("KDMX20220305_120000_V06"));
    }
}
