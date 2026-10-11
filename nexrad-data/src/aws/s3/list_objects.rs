use crate::aws::client::client;
use crate::aws::s3::bucket_list_result::BucketListResult;
use crate::aws::s3::bucket_object::BucketObject;
use crate::aws::s3::bucket_object_field::BucketObjectField;
use crate::result::aws::AWSError;
use crate::result::aws::AWSError::S3ListObjects;
use chrono::{DateTime, Utc};
use log::{debug, trace, warn};
use xml::reader::XmlEvent;
use xml::EventReader;

/// Lists objects from a S3 bucket with the specified prefix. A maximum number of keys can be
/// specified to limit the number of objects returned, otherwise it will use AWS's default (1000).
pub async fn list_objects(
    bucket: &str,
    prefix: &str,
    max_keys: Option<usize>,
) -> crate::result::Result<BucketListResult> {
    list_objects_page(bucket, prefix, max_keys, None).await
}

async fn list_objects_page(
    bucket: &str,
    prefix: &str,
    max_keys: Option<usize>,
    after: Option<String>,
) -> crate::result::Result<BucketListResult> {
    let mut path = format!("https://{bucket}.s3.amazonaws.com?list-type=2&prefix={prefix}");
    if let Some(max_keys) = max_keys {
        path.push_str(&format!("&max-keys={max_keys}"));
    }
    debug!("Listing objects in bucket \"{bucket}\" with prefix \"{prefix}\"");

    let mut url = reqwest::Url::parse(&path).map_err(|_| AWSError::S3ListObjectsDecoding)?;
    if let Some(after) = after {
        url.query_pairs_mut().append_pair("start-after", &after);
    }
    let response = client()
        .get(url)
        .send()
        .await
        .map_err(S3ListObjects)?
        .error_for_status()
        .map_err(S3ListObjects)?;
    trace!("  List objects response status: {}", response.status());

    let body = response.text().await.map_err(S3ListObjects)?;
    trace!("  List objects response body length: {}", body.len());

    let parser = EventReader::new(body.as_bytes());

    let mut objects = Vec::new();
    let mut truncated = false;
    let mut object: Option<BucketObject> = None;

    let mut field: Option<BucketObjectField> = None;
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => match name.local_name.as_ref() {
                "IsTruncated" => field = Some(BucketObjectField::IsTruncated),
                "Contents" => {
                    object = Some(BucketObject {
                        key: String::new(),
                        last_modified: None,
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
                    if field == &BucketObjectField::IsTruncated {
                        truncated = chars == "true";
                        if truncated {
                            trace!("  List objects truncated: {truncated}");
                        }
                        continue;
                    }

                    let item = object.as_mut().ok_or_else(|| {
                        warn!("Expected item for object field: {field:?}");
                        AWSError::S3ListObjectsDecoding
                    })?;
                    match field {
                        BucketObjectField::Key => item.key.push_str(&chars),
                        BucketObjectField::LastModified => {
                            item.last_modified = DateTime::parse_from_rfc3339(&chars)
                                .ok()
                                .map(|date_time| date_time.with_timezone(&Utc));
                        }
                        BucketObjectField::Size => {
                            item.size = chars.parse().map_err(|_| {
                                warn!("Error parsing object size: {chars}");
                                AWSError::S3ListObjectsDecoding
                            })?;
                        }
                        _ => {}
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) if name.local_name.as_str() == "Contents" => {
                if let Some(item) = object.take() {
                    objects.push(item);
                }
            }
            _ => {}
        }
    }

    trace!("  List objects found: {}", objects.len());

    Ok(BucketListResult { truncated, objects })
}

/// List every page; current radar scans may sort after retained old scans.
pub(crate) async fn list_all_objects(
    bucket: &str,
    prefix: &str,
) -> crate::result::Result<Vec<BucketObject>> {
    collect_pages(|after| async move { list_objects_page(bucket, prefix, None, after).await }).await
}

async fn collect_pages<F: std::future::Future<Output = crate::result::Result<BucketListResult>>>(
    mut fetch: impl FnMut(Option<String>) -> F,
) -> crate::result::Result<Vec<BucketObject>> {
    let mut objects = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let page = fetch(after.clone()).await?;
        if page.truncated {
            let last = page.objects.last().ok_or(AWSError::S3ListObjectsDecoding)?;
            if after.as_ref().is_some_and(|previous| last.key <= *previous) {
                return Err(AWSError::S3ListObjectsDecoding.into());
            }
            after = Some(last.key.clone());
        }
        objects.extend(page.objects);
        if !page.truncated {
            return Ok(objects);
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    fn page(key: &str, truncated: bool) -> BucketListResult {
        BucketListResult {
            truncated,
            objects: vec![BucketObject {
                key: key.into(),
                last_modified: None,
                size: 0,
            }],
        }
    }
    #[tokio::test]
    async fn reads_new_scan_after_old_scan_fills_first_page() {
        let objects = collect_pages(|after| async move {
            match after.as_deref() {
                None => Ok(page("KBYX/589/20260909-013029-100-I", true)),
                Some("KBYX/589/20260909-013029-100-I") => {
                    Ok(page("KBYX/589/20260912-140452-001-S", false))
                }
                _ => panic!("unexpected pagination cursor"),
            }
        })
        .await
        .unwrap();
        assert_eq!(objects.len(), 2);
        assert!(objects[1].key.contains("20260912"));
    }
    #[tokio::test]
    async fn rejects_truncated_page_without_progress() {
        let result = collect_pages(|_| async { Ok(page("same-key", true)) }).await;
        assert!(result.is_err());
    }
}
