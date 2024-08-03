//!
//! Downloads NEXRAD level-II data from an AWS S3 bucket populated by NOAA.
//!

pub mod archive;

use chrono::{DateTime, NaiveDate, Utc};
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::file::FileMetadata;
use crate::result::{Error, Result};

const ARCHIVE_BUCKET: &str = "noaa-nexrad-level2";

/// List data files for the specified site and date. This effectively returns an index of data files
/// which can then be individually downloaded.
pub async fn list_files(site: &str, date: &NaiveDate) -> Result<Vec<FileMetadata>> {
    let prefix = format!("{}/{}", date.format("%Y/%m/%d"), site);
    let objects = list_objects(ARCHIVE_BUCKET, &prefix).await?;

    let metas = objects
        .iter()
        .map(|object| {
            // E.g. 2023/04/06/KDMX/KDMX20230406_000215_V06
            //      date_string:    "2023_04_06"
            //      site:           "KDMX"
            //      identifier:     "KDMX20230406_000215_V06"
            let parts: Vec<&str> = object.key.split('/').collect();

            let date_string = parts[0..=2].join("/");
            let date = NaiveDate::parse_from_str(&date_string, "%Y/%m/%d")
                .unwrap_or_else(|_| panic!("file has valid date: \"{date_string}\""));

            let site = parts[3];
            let identifier = parts[4..].join("");

            FileMetadata::new(site.to_string(), date, identifier)
        })
        .collect();

    Ok(metas)
}

/// Download a data file specified by its metadata. Returns the downloaded file's encoded contents
/// which may then need to be decompressed and decoded.
pub async fn download_file(meta: &FileMetadata) -> Result<Vec<u8>> {
    let formatted_date = meta.date().format("%Y/%m/%d");
    let key = format!("{}/{}/{}", formatted_date, meta.site(), meta.identifier());

    download_object(ARCHIVE_BUCKET, &key).await
}

/// Downloads an object from S3 and returns only its contents. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn download_object(bucket: &str, key: &str) -> Result<Vec<u8>> {
    let path = format!("https://{bucket}.s3.amazonaws.com/{key}");
    let response = reqwest::get(path).await.map_err(Error::S3GetObjectError)?;

    let bytes = response.bytes().await.map_err(Error::S3StreamingError)?;
    Ok(bytes.to_vec())
}

/// Lists objects from a S3 bucket with the specified prefix. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn list_objects(bucket: &str, prefix: &str) -> Result<Vec<BucketObject>> {
    let path = format!("https://{bucket}.s3.amazonaws.com?prefix={prefix}");
    let response = reqwest::get(path)
        .await
        .map_err(Error::S3ListObjectsError)?;

    let body = response.text().await.map_err(Error::S3ListObjectsError)?;
    let parser = EventReader::new(body.as_bytes());

    let mut objects = Vec::new();
    let mut object: Option<BucketObject> = None;

    let mut field: Option<BucketObjectField> = None;
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => match name.local_name.as_ref() {
                "Contents" => {
                    object = Some(BucketObject {
                        key: String::new(),
                        last_modified: Utc::now(),
                        size: 0,
                    });
                }
                "Key" => field = Some(BucketObjectField::Key),
                "LastModified" => field = Some(BucketObjectField::LastModified),
                "Size" => field = Some(BucketObjectField::Size),
                _ => field = None,
            },
            Ok(XmlEvent::Characters(chars)) => {
                if let Some(field) = field.as_ref() {
                    let item = object.as_mut().expect("item should exist");
                    match field {
                        BucketObjectField::Key => item.key.push_str(&chars),
                        BucketObjectField::LastModified => {
                            item.last_modified = DateTime::parse_from_rfc3339(&chars)
                                .expect("should parse date")
                                .with_timezone(&Utc);
                        }
                        BucketObjectField::Size => {
                            item.size = chars.parse().expect("should parse size")
                        }
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name.as_str() == "Contents" {
                    if let Some(item) = object.take() {
                        objects.push(item);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(objects)
}

/// A field describing an S3 bucket object.
enum BucketObjectField {
    Key,
    LastModified,
    Size,
}

/// A bucket object returned from an S3 list objects request.
struct BucketObject {
    key: String,
    last_modified: DateTime<Utc>,
    size: u64,
}
