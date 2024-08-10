use std::future::Future;

/// Performs a binary search from `min` to `max` inclusive to find the index with the greatest
/// value. If no values are found, `None` is returned.
pub(crate) async fn binary_search_greatest<F, V>(
    min: usize,
    max: usize,
    mut f: impl FnMut(usize) -> F,
) -> crate::result::Result<Option<usize>>
where
    F: Future<Output = crate::result::Result<Option<V>>>,
    V: PartialOrd + Clone,
{
    let mut low = min;
    let mut low_value = f(low).await?;
    if low_value.is_none() {
        let original_low = low;
        low += 1;
        let mut high = max + 1;
        while low < high {
            let mid = low + (high - low) / 2;
            match f(mid).await? {
                Some(_) => high = mid,
                None => low = mid + 1,
            };
        }
        if low != original_low {
            low_value = f(low).await?;
        }
    }

    if low_value.is_none() {
        return Ok(None);
    }

    let mut high = max + 1;
    while low < high {
        let mid = low + (high - low) / 2;

        match f(mid).await? {
            None => {
                high = mid;
                continue;
            }
            Some(mid_value) => {
                if &mid_value < low_value.as_ref().unwrap() {
                    high = mid;
                    continue;
                }
            }
        }

        low = mid + 1;
    }

    Ok(Some(low - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod binary_search_greatest {
        use super::*;
        use chrono::{DateTime, NaiveDateTime, Utc};

        // Volumes are one-indexed in AWS, so apply an offset to assert indexes are shifted properly
        const VOLUME_OFFSET: usize = 0;

        macro_rules! assert_binary_search_greatest {
            ($test_cases:expr, $expected:expr) => {
                let mut end = VOLUME_OFFSET;
                if $test_cases.len() > 0 {
                    end += $test_cases.len() - 1;
                }

                let result = binary_search_greatest(VOLUME_OFFSET, end, |i| async move {
                    println!("Testing index {}", i);
                    Ok($test_cases.get(i - VOLUME_OFFSET).cloned().flatten())
                })
                .await;

                assert!(result.is_ok());
                assert_eq!(result.unwrap(), $expected);
            };
        }

        macro_rules! date_time {
            ($date_time_string:expr) => {
                Some(
                    NaiveDateTime::parse_from_str($date_time_string, "%Y/%m/%d %H:%M:%S")
                        .unwrap()
                        .and_utc(),
                )
            };
        }

        #[tokio::test]
        async fn test_incrementing() {
            assert_binary_search_greatest!(
                [
                    date_time!("2024/08/03 00:00:00"),
                    date_time!("2024/08/03 01:00:00"),
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                    date_time!("2024/08/04 01:00:00"),
                    date_time!("2024/08/04 02:00:00"),
                ],
                Some(VOLUME_OFFSET + 5)
            );
        }

        #[tokio::test]
        async fn test_end_gap() {
            assert_binary_search_greatest!(
                [
                    date_time!("2024/08/03 00:00:00"),
                    date_time!("2024/08/03 01:00:00"),
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                    None,
                    None,
                ],
                Some(VOLUME_OFFSET + 3)
            );
        }

        #[tokio::test]
        async fn test_start_gap() {
            assert_binary_search_greatest!(
                [
                    None,
                    None,
                    date_time!("2024/08/03 00:00:00"),
                    date_time!("2024/08/03 01:00:00"),
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                ],
                Some(VOLUME_OFFSET + 5)
            );
        }

        #[tokio::test]
        async fn test_start_end_gaps() {
            assert_binary_search_greatest!(
                [
                    None,
                    date_time!("2024/08/03 00:00:00"),
                    date_time!("2024/08/03 01:00:00"),
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                    None,
                ],
                Some(VOLUME_OFFSET + 4)
            );
        }

        #[tokio::test]
        async fn test_wrapping() {
            assert_binary_search_greatest!(
                [
                    date_time!("2024/08/04 01:00:00"),
                    date_time!("2024/08/04 02:00:00"),
                    date_time!("2024/08/03 00:00:00"),
                    date_time!("2024/08/03 01:00:00"),
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                ],
                Some(VOLUME_OFFSET + 1)
            );
        }

        #[tokio::test]
        async fn test_wrapping_gap() {
            assert_binary_search_greatest!(
                [
                    date_time!("2024/08/04 01:00:00"),
                    date_time!("2024/08/04 02:00:00"),
                    None,
                    None,
                    date_time!("2024/08/03 02:00:00"),
                    date_time!("2024/08/04 00:00:00"),
                ],
                Some(VOLUME_OFFSET + 1)
            );
        }

        #[tokio::test]
        async fn test_empty_list() {
            let empty: [Option<DateTime<Utc>>; 0] = [];
            assert_binary_search_greatest!(empty, None);
        }

        #[tokio::test]
        async fn test_single_list() {
            assert_binary_search_greatest!(
                [date_time!("2024/08/04 00:00:00")],
                Some(VOLUME_OFFSET)
            );
        }

        #[tokio::test]
        async fn test_list_of_empty() {
            let empties: [Option<DateTime<Utc>>; 5] = [None; 5];
            assert_binary_search_greatest!(empties, None);
        }
    }
}
