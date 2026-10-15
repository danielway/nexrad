#![cfg(feature = "aws")]

use chrono::NaiveDate;
use nexrad_data::aws::archive::{self, Identifier};

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_files_basic() {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();

    let result = archive::list_files(site, date).await;

    assert!(result.is_ok(), "Failed to list files: {:?}", result.err());
    let files = result.unwrap();

    // There should be files for this date/site (bucket may be empty during testing)
    if !files.is_empty() {
        // All files should have the correct site
        for file in &files {
            assert_eq!(file.site(), site);
            assert!(!file.is_metadata());
        }
    } else {
        println!(
            "Warning: No files found for KDMX on 2022-03-05, bucket may be empty or inaccessible"
        );
    }
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_files_with_sizes_matches_list_files() {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();

    let names: Vec<String> = archive::list_files(site, date)
        .await
        .unwrap()
        .iter()
        .map(|file| file.name().to_string())
        .collect();
    let listed = archive::list_files_with_sizes(site, date).await.unwrap();

    let listed_names: Vec<String> = listed
        .iter()
        .map(|file| file.identifier().name().to_string())
        .collect();
    assert_eq!(names, listed_names);
    for file in &listed {
        assert!(!file.identifier().is_metadata());
        assert!(file.size() > 0, "{} has no size", file.identifier().name());
    }
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_files_no_data() {
    let site = "KDMX";
    // Use a date far in the future where no data should exist
    let date = NaiveDate::from_ymd_opt(2099, 12, 31).unwrap();

    let result = archive::list_files(site, date).await;

    assert!(result.is_ok());
    let files = result.unwrap();

    // Should return empty list for future date
    assert!(
        files.is_empty(),
        "Expected no files for future date, got: {}",
        files.len()
    );
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_files_sorted() {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();

    let result = archive::list_files(site, date).await;
    assert!(result.is_ok());
    let files = result.unwrap();

    if files.len() > 1 {
        // Files should be sorted by time
        for i in 0..files.len() - 1 {
            assert!(
                files[i].date_time() <= files[i + 1].date_time(),
                "Files not sorted: {} comes after {}",
                files[i].name(),
                files[i + 1].name()
            );
        }
    }
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_download_file_basic() {
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();

    // First list files to get an actual file that exists
    let list_result = archive::list_files(site, date).await;
    assert!(list_result.is_ok(), "Failed to list files");
    let files = list_result.unwrap();

    if files.is_empty() {
        // Skip test if no files are available
        println!("No files available for testing, skipping");
        return;
    }

    // Download the first available file
    let identifier = files.first().unwrap().clone();
    let result = archive::download_file(&identifier).await;

    assert!(
        result.is_ok(),
        "Failed to download file {}: {:?}",
        identifier.name(),
        result.err()
    );
    let file = result.unwrap();

    // File should have data
    assert!(!file.data().is_empty(), "Downloaded file has no data");

    // File should start with AR2 header
    assert!(
        file.data().starts_with(b"AR2"),
        "File doesn't start with AR2 header"
    );
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_download_file_nonexistent() {
    // Use an identifier that should not exist
    let identifier = Identifier::parse("KDMX20990101_000000_V06").unwrap();

    let result = archive::download_file(&identifier).await;

    // Should return an error for non-existent file
    assert!(result.is_err(), "Expected error for non-existent file");
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_and_download_workflow() {
    // Test a complete workflow: list files, then download one
    let site = "KDMX";
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();

    // List files
    let list_result = archive::list_files(site, date).await;
    assert!(list_result.is_ok());
    let files = list_result.unwrap();

    if files.is_empty() {
        println!("No files available for testing workflow, skipping");
        return;
    }

    // Download the first file
    let first_file = files.first().unwrap().clone();
    let download_result = archive::download_file(&first_file).await;
    assert!(
        download_result.is_ok(),
        "Failed to download: {:?}",
        download_result.err()
    );

    let file = download_result.unwrap();

    // Verify the file content matches expectations
    assert!(!file.data().is_empty());
    assert!(file.data().starts_with(b"AR2"));

    // Verify we can get records from the downloaded file
    let records = file.records().expect("records");
    assert!(!records.is_empty(), "File should contain records");
}

#[tokio::test]
#[ignore = "requires AWS access"]
async fn test_list_files_different_sites() {
    let date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    let sites = vec!["KDMX", "KABR"];

    for site in sites {
        let result = archive::list_files(site, date).await;
        assert!(result.is_ok(), "Failed to list files for site: {}", site);

        let files = result.unwrap();
        // All returned files should be for the requested site
        for file in files {
            assert_eq!(file.site(), site, "File {} has wrong site", file.name());
        }
    }
}
