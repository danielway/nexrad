use std::collections::VecDeque;
use std::future::Future;

/// Finds the index of the greatest value in a circular sequence of `element_count` elements.
///
/// The sequence is expected to ascend in circular order, with at most one contiguous run of `None`
/// elements separating the greatest value from the smallest. This matches how the real-time bucket
/// rotates through its volumes: following the newest volume are any not yet populated, then the
/// oldest. `f` is called with an index and resolves to that element's value, or `None` if the
/// element is empty. Returns `None` if every element is empty.
///
/// The sequence is probed `O(log element_count)` times: a few probes to locate any populated
/// element, then one binary search.
pub(crate) async fn find_latest<F, V>(
    element_count: usize,
    mut f: impl FnMut(usize) -> F,
) -> crate::result::Result<Option<usize>>
where
    F: Future<Output = crate::result::Result<Option<V>>>,
    V: PartialOrd,
{
    let Some((anchor, anchor_value)) = find_populated(element_count, &mut f).await? else {
        return Ok(None);
    };

    // Walking the sequence circularly from the anchor, the elements whose value is at least the
    // anchor's form a prefix that ends at the greatest value. The gap and the smaller values follow,
    // so a binary search for the end of that prefix finds the greatest value.
    //
    // Invariant: the prefix includes offset `low` and excludes offset `high`.
    let mut low = 0;
    let mut high = element_count;
    while high - low > 1 {
        let mid = low + (high - low) / 2;
        let value = f((anchor + mid) % element_count).await?;
        if value.is_some_and(|value| value >= anchor_value) {
            low = mid;
        } else {
            high = mid;
        }
    }

    Ok(Some((anchor + low) % element_count))
}

/// Locates any populated element, returning its index and value. Probes index 0 and then bisects
/// the sequence breadth-first, so a single contiguous gap covering less than half of the sequence
/// costs at most two probes.
async fn find_populated<F, V>(
    element_count: usize,
    f: &mut impl FnMut(usize) -> F,
) -> crate::result::Result<Option<(usize, V)>>
where
    F: Future<Output = crate::result::Result<Option<V>>>,
{
    if element_count == 0 {
        return Ok(None);
    }

    if let Some(value) = f(0).await? {
        return Ok(Some((0, value)));
    }

    let mut queue = VecDeque::from([(0, element_count)]);
    while let Some((start, end)) = queue.pop_front() {
        if end - start < 2 {
            continue;
        }

        let mid = start + (end - start) / 2;
        if let Some(value) = f(mid).await? {
            return Ok(Some((mid, value)));
        }

        queue.push_back((start, mid));
        queue.push_back((mid, end));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Runs `find_latest` over an in-memory sequence, returning the result and the probe count.
    async fn run(elements: &[Option<usize>]) -> (Option<usize>, usize) {
        let probes = Cell::new(0);
        let result = find_latest(elements.len(), |i| {
            probes.set(probes.get() + 1);
            let value = elements[i];
            async move { Ok(value) }
        })
        .await
        .unwrap();
        (result, probes.get())
    }

    /// Builds a sequence of `populated` ascending values followed by `empty` gaps, rotated so the
    /// element originally at index `rotation` comes first.
    fn sequence(populated: usize, empty: usize, rotation: usize) -> Vec<Option<usize>> {
        let mut base: Vec<_> = (0..populated).map(Some).collect();
        base.extend(std::iter::repeat_n(None, empty));
        let len = base.len();
        (0..len).map(|i| base[(i + rotation) % len]).collect()
    }

    fn index_of_max(elements: &[Option<usize>]) -> Option<usize> {
        elements
            .iter()
            .enumerate()
            .filter_map(|(i, value)| value.map(|value| (value, i)))
            .max()
            .map(|(_, i)| i)
    }

    #[tokio::test]
    async fn empty_sequence() {
        assert_eq!(run(&[]).await.0, None);
    }

    #[tokio::test]
    async fn all_elements_empty() {
        assert_eq!(run(&[None, None, None]).await.0, None);
    }

    #[tokio::test]
    async fn single_element() {
        assert_eq!(run(&[Some(7)]).await.0, Some(0));
    }

    #[tokio::test]
    async fn greatest_value_first() {
        assert_eq!(run(&[Some(1), Some(0)]).await.0, Some(0));
    }

    #[tokio::test]
    async fn greatest_value_last() {
        assert_eq!(run(&[Some(0), Some(1)]).await.0, Some(1));
    }

    #[tokio::test]
    async fn greatest_value_before_gap() {
        let elements = [Some(5), Some(6), Some(7), None, None, Some(1), Some(2)];
        assert_eq!(run(&elements).await.0, Some(2));
    }

    #[tokio::test]
    async fn gap_wraps_around_end() {
        let elements = [None, Some(3), Some(4), Some(5), Some(6), None, None];
        assert_eq!(run(&elements).await.0, Some(4));
    }

    #[tokio::test]
    async fn gap_covers_most_of_sequence() {
        let mut elements = vec![None; 998];
        elements[600] = Some(1);
        elements[601] = Some(2);
        assert_eq!(run(&elements).await.0, Some(601));
    }

    /// Exhaustively checks every rotation of every small combination of populated and empty counts.
    #[tokio::test]
    async fn every_rotation_of_small_sequences() {
        for populated in 1..24 {
            for empty in 0..24 {
                let len = populated + empty;
                for rotation in 0..len {
                    let elements = sequence(populated, empty, rotation);
                    let (result, _) = run(&elements).await;
                    assert_eq!(
                        result,
                        index_of_max(&elements),
                        "populated={populated} empty={empty} rotation={rotation}"
                    );
                }
            }
        }
    }

    /// Guards the network cost of `get_latest_volume`: with the bucket's 998 volumes and a gap no
    /// larger than half of them, locating a populated volume takes at most two probes and the binary
    /// search at most ten, so the total must not exceed twelve.
    #[tokio::test]
    async fn probe_count_is_bounded_for_realistic_sequences() {
        for empty in [0, 1, 200, 499] {
            for rotation in (0..998).step_by(7) {
                let elements = sequence(998 - empty, empty, rotation);
                let (result, probes) = run(&elements).await;
                assert_eq!(result, index_of_max(&elements));
                assert!(
                    probes <= 12,
                    "{probes} probes with empty={empty} rotation={rotation}"
                );
            }
        }
    }
}
