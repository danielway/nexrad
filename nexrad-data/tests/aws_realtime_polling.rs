#![cfg(feature = "aws")]

// Integration tests that require network access
#[cfg(all(feature = "aws", not(target_arch = "wasm32")))]
mod polling_tests {
    use futures_util::StreamExt;
    use nexrad_data::aws::realtime::{chunk_stream, PollConfig};
    use std::pin::pin;
    use std::time::Duration as StdDuration;

    #[tokio::test]
    #[ignore = "requires AWS access and takes time"]
    async fn test_chunk_stream_basic() {
        let config = PollConfig::new("KDMX");
        let stream = chunk_stream(config, |d| tokio::time::sleep(d));
        let mut stream = pin!(stream);

        let mut chunk_count = 0;

        // Use timeout to limit test duration
        let result = tokio::time::timeout(StdDuration::from_secs(30), async {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(downloaded) => {
                        assert!(!downloaded.identifier.name().is_empty());
                        assert!(!downloaded.chunk.data().is_empty());
                        chunk_count += 1;

                        // Stop after receiving a few chunks
                        if chunk_count >= 3 {
                            break;
                        }
                    }
                    Err(e) => {
                        panic!("Stream error: {:?}", e);
                    }
                }
            }
        })
        .await;

        assert!(result.is_ok(), "Test timed out");
        assert!(chunk_count > 0, "Expected to receive at least one chunk");
    }
}
