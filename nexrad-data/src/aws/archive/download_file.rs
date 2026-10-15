use crate::aws::archive::identifier::Identifier;
use crate::aws::archive::ARCHIVE_BUCKET;
use crate::aws::s3::download_object;
use crate::volume::File;

/// Download a data file specified by its metadata. Returns the downloaded file's encoded contents
/// which may then need to be decompressed and decoded.
pub async fn download_file(identifier: &Identifier) -> crate::result::Result<File> {
    let key = format!(
        "{}/{}/{}",
        identifier.date_time().format("%Y/%m/%d"),
        identifier.site(),
        identifier.name()
    );
    let downloaded_object = download_object(ARCHIVE_BUCKET, &key).await?;

    Ok(File::new(downloaded_object.data))
}
