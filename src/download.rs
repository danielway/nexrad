//!
//! Downloads NEXRAD level-II data from an AWS S3 bucket populated by NOAA.
//!

use aws_sdk_s3::{config::Region, types::Object, Client, Config};
use chrono::NaiveDate;

use crate::file::FileMetadata;
use crate::result::Result;

const REGION: &str = "us-east-1";
const BUCKET: &str = "noaa-nexrad-level2";

/// List data files for the specified site and date. This effectively returns an index of data files
/// which can then be individually downloaded.
pub async fn list_files(site: &str, date: &NaiveDate) -> Result<Vec<FileMetadata>> {
    // Query S3 for objects matching the prefix (i.e. files for the specified site and date)
    let prefix = format!("{}/{}", date.format("%Y/%m/%d"), site);
    let objects = list_objects(&get_client().await, BUCKET, &prefix)
        .await?
        .expect("should return objects");

    // Pull the returned objects' keys and parse them into metadata
    let metas = objects
        .iter()
        .map(|object| {
            let key = object.key().expect("object should have a key");

            // E.g. 2023/04/06/KDMX/KDMX20230406_000215_V06
            //      date_string:    "2023_04_06"
            //      site:           "KDMX"
            //      identifier:     "KDMX20230406_000215_V06"

            let parts: Vec<&str> = key.split("/").collect();

            let date_string = parts[0..=2].join("/");
            let date = NaiveDate::parse_from_str(&date_string, "%Y/%m/%d")
                .expect(&format!("file has valid date: \"{}\"", date_string));

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
    // Reconstruct the S3 object key from the file's metadata
    let formatted_date = meta.date().format("%Y/%m/%d");
    let key = format!("{}/{}/{}", formatted_date, meta.site(), meta.identifier());

    // Download the object from S3
    Ok(download_object(&get_client().await, BUCKET, &key).await?)
}

/// Downloads an object from S3 and returns only its contents. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn download_object(client: &Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    let operation = client.get_object().bucket(bucket).key(key);

    let response = operation.send().await?;
    let bytes = response.body.collect().await?;

    Ok(bytes.to_vec())
}

/// Lists objects from a S3 bucket with the specified prefix. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn list_objects(client: &Client, bucket: &str, prefix: &str) -> Result<Option<Vec<Object>>> {
    let operation = client.list_objects_v2().bucket(bucket).prefix(prefix);

    let response = operation.send().await?;
    Ok(response.contents().map(|objects| objects.to_vec()))
}

/// Creates a new S3 client for a predetermined region.
async fn get_client() -> Client {
    Client::from_conf(
        Config::builder()
            .region(Region::from_static(REGION))
            .build()
    )
}
