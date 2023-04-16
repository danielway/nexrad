use aws_config::from_env;
use aws_sdk_s3::{Client, config::Region, types::Object};
use aws_sig_auth::signer::{OperationSigningConfig, SigningRequirements};
use aws_smithy_http::operation::Operation;
use chrono::{Datelike, NaiveDate};

use crate::chunk::{ChunkMeta, EncodedChunk};
use crate::result::Result;

const REGION: &str = "us-east-1";
const BUCKET: &str = "noaa-nexrad-level2";

/// List chunk metas for the specified site and date. This effectively returns an index of chunks
/// which can then be individually fetched/downloaded.
pub async fn list_chunks(site: &str, date: &NaiveDate) -> Result<Vec<ChunkMeta>> {
    // Query S3 for objects matching the prefix (i.e. chunks for the specified site and date)
    let client = get_client().await;
    let objects = list_objects(&client, BUCKET, &get_bucket_prefix(date, site)).await?
        .expect("should return objects");

    // Pull the returned objects' keys and parse them into chunk metas
    let metas = objects.iter().map(|object| {
        let key = object.key().expect("object should have a key");

        // Ex. 2023/04/06/KDMX/KDMX20230406_000215_V06
        //      date_string:    "2023_04_06"
        //      site:           "KDMX"
        //      identifier:     "KDMX20230406_000215_V06"

        let parts: Vec<&str> = key.split("/").collect();

        let date_string = parts[0..=2].join("/");
        let date = NaiveDate::parse_from_str(&date_string, "%Y/%m/%d")
            .expect(&format!("chunk has valid date: \"{}\"", date_string));

        let site = parts[3];
        let identifier = parts[4..].join("");

        ChunkMeta::new(site.to_string(), date, identifier)
    }).collect();

    Ok(metas)
}

/// Fetch/download a chunk specified by its meta. This downloads and returns the chunk's encoded
/// contents which may then need to be decompressed and decoded.
pub async fn fetch_chunk(meta: &ChunkMeta) -> Result<EncodedChunk> {
    // Reconstruct the S3 object key from the chunk meta
    let prefix = get_bucket_prefix(meta.date(), meta.site());
    let key = format!("{}/{}", prefix, meta.identifier());

    // Fetch the object from S3
    let client = get_client().await;
    let data = get_object(&client, BUCKET, &key).await?;

    // Wrap the object contents in an EncodedChunk
    Ok(EncodedChunk::new(meta.clone(), data))
}

/// Returns the bucket prefix corresponding to the specified site and date.
fn get_bucket_prefix(date: &NaiveDate, site: &str) -> String {
    // Pad month/date with leading zeros, Ex. 2023/04/06 instead of 2023/4/6
    let date_prefix = format!("{}/{:0>2}/{:0>2}", date.year(), date.month(), date.day());

    // Ex. 2023/04/06/KDMX
    format!("{}/{}", date_prefix, site)
}

/// Fetches an object from S3 and returns only its contents. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn get_object(client: &Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    let builder = client
        .get_object()
        .bucket(bucket)
        .key(key);

    let operation = builder.customize().await?.map_operation(make_unsigned).unwrap();

    let response = operation.send().await?;
    let bytes = response.body.collect().await.unwrap();

    Ok(bytes.to_vec())
}

/// Lists objects from a S3 bucket with the specified prefix. This will only work for
/// unauthenticated requests (requests are unsigned).
async fn list_objects(client: &Client, bucket: &str, prefix: &str) -> Result<Option<Vec<Object>>> {
    let builder = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix);

    let operation = builder.customize().await?.map_operation(make_unsigned).unwrap();
    let response = operation.send().await?;

    Ok(response.contents().map(|objects| objects.to_vec()))
}

/// Creates a new S3 client for a predetermined region.
async fn get_client() -> Client {
    let conf = from_env().region(Region::new(REGION)).load().await;
    Client::new(&conf)
}

/// Disables signing requirements for unauthenticated S3 requests.
fn make_unsigned<O, Retry>(
    mut operation: Operation<O, Retry>,
) -> std::result::Result<Operation<O, Retry>, std::convert::Infallible> {
    {
        let mut props = operation.properties_mut();
        let mut signing_config = props
            .get_mut::<OperationSigningConfig>()
            .expect("has signing_config");
        signing_config.signing_requirements = SigningRequirements::Disabled;
    }

    Ok(operation)
}