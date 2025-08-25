use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

// Tests for skip method
#[tokio::test]
async fn skip_empty_stream() {
    let result: Vec<i32> = stream::empty::<i32>().skip(5).collect().await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn skip_zero_elements() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).skip(0).collect().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn skip_some_elements() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).skip(2).collect().await;
    assert_eq!(result, vec![3, 4, 5]);
}

#[tokio::test]
async fn skip_all_elements() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).skip(5).collect().await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn skip_more_than_stream_length() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3]).skip(10).collect().await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn skip_single_element_stream() {
    let result: Vec<i32> = stream::iter(vec![42]).skip(1).collect().await;
    assert!(result.is_empty());

    let result: Vec<i32> = stream::iter(vec![42]).skip(0).collect().await;
    assert_eq!(result, vec![42]);
}

#[tokio::test]
async fn skip_first_element_only() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).skip(1).collect().await;
    assert_eq!(result, vec![2, 3, 4, 5]);
}

#[tokio::test]
async fn skip_large_number_of_elements() {
    let input: Vec<i32> = (0..100).collect();
    let result: Vec<i32> = stream::iter(input.clone()).skip(90).collect().await;
    assert_eq!(result, vec![90, 91, 92, 93, 94, 95, 96, 97, 98, 99]);
}

#[tokio::test]
async fn skip_with_strings() {
    let result: Vec<&str> = stream::iter(vec!["first", "second", "third", "fourth"])
        .skip(2)
        .collect()
        .await;
    assert_eq!(result, vec!["third", "fourth"]);
}

#[tokio::test]
async fn skip_debug_format() {
    let skip_stream = stream::iter(vec![1, 2, 3, 4, 5]).skip(2);
    let debug_str = format!("{skip_stream:?}");
    assert!(debug_str.contains("Skip"));
}

#[tokio::test]
async fn skip_size_hint_exact() {
    let skip_stream = stream::iter(vec![1, 2, 3, 4, 5]).skip(2);
    let (lower, upper) = skip_stream.size_hint();
    assert_eq!(lower, 3); // 5 - 2
    assert_eq!(upper, Some(3)); // 5 - 2
}

#[tokio::test]
async fn skip_size_hint_skip_more_than_length() {
    let skip_stream = stream::iter(vec![1, 2, 3]).skip(5);
    let (lower, upper) = skip_stream.size_hint();
    assert_eq!(lower, 0); // saturating_sub
    assert_eq!(upper, Some(0)); // saturating_sub
}

#[tokio::test]
async fn skip_size_hint_unknown_upper_bound() {
    // Use a stream with unknown size - test the size hint calculation
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    drop(tx);

    let original_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    let (orig_lower, orig_upper) = original_stream.size_hint();

    let skip_stream = original_stream.skip(1);
    let (lower, upper) = skip_stream.size_hint();

    // The lower bound should be the original lower bound minus skip amount (saturating)
    assert_eq!(lower, orig_lower.saturating_sub(1));
    // The upper bound should follow the same pattern
    assert_eq!(upper, orig_upper.map(|x| x.saturating_sub(1)));
}

#[tokio::test]
async fn skip_pending_stream() {
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        tx.send(4).unwrap();
        tx.send(5).unwrap();
        drop(tx); // Close the stream

        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .skip(2)
            .collect::<Vec<_>>()
            .await
    });

    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![3, 4, 5]); // First two skipped
}

#[tokio::test]
async fn skip_chaining() {
    // Test chaining multiple skip operations
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .skip(3) // Skip 1, 2, 3 -> [4, 5, 6, 7, 8, 9, 10]
        .skip(2) // Skip 4, 5 -> [6, 7, 8, 9, 10]
        .collect()
        .await;
    assert_eq!(result, vec![6, 7, 8, 9, 10]);
}

#[tokio::test]
async fn skip_with_other_adaptors() {
    // Test skip combined with other stream adaptors
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8])
        .skip(2) // Skip 1, 2 -> [3, 4, 5, 6, 7, 8]
        .filter(|&x| x % 2 == 0) // Even numbers -> [4, 6, 8]
        .collect()
        .await;
    assert_eq!(result, vec![4, 6, 8]);
}

#[tokio::test]
async fn skip_then_take() {
    // Common pattern: skip some, take some
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .skip(3) // Skip first 3 -> [4, 5, 6, 7, 8, 9, 10]
        .take(4) // Take next 4 -> [4, 5, 6, 7]
        .collect()
        .await;
    assert_eq!(result, vec![4, 5, 6, 7]);
}

#[tokio::test]
async fn skip_different_types() {
    // Test with different data types
    let result: Vec<(i32, String)> = stream::iter(vec![
        (1, "one".to_string()),
        (2, "two".to_string()),
        (3, "three".to_string()),
        (4, "four".to_string()),
    ])
    .skip(1)
    .collect()
    .await;

    assert_eq!(
        result,
        vec![
            (2, "two".to_string()),
            (3, "three".to_string()),
            (4, "four".to_string()),
        ]
    );
}

#[tokio::test]
async fn skip_pagination_pattern() {
    // Common pagination pattern
    let data: Vec<i32> = (1..=20).collect();

    // Page 1: skip 0, take 5
    let page1: Vec<i32> = stream::iter(data.clone()).skip(0).take(5).collect().await;
    assert_eq!(page1, vec![1, 2, 3, 4, 5]);

    // Page 2: skip 5, take 5
    let page2: Vec<i32> = stream::iter(data.clone()).skip(5).take(5).collect().await;
    assert_eq!(page2, vec![6, 7, 8, 9, 10]);

    // Page 3: skip 10, take 5
    let page3: Vec<i32> = stream::iter(data.clone()).skip(10).take(5).collect().await;
    assert_eq!(page3, vec![11, 12, 13, 14, 15]);
}

#[tokio::test]
async fn skip_early_termination_stream() {
    // Test skip with a stream that might terminate early
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .take_while(|&x| x < 4) // [1, 2, 3]
        .skip(1) // Skip 1 -> [2, 3]
        .collect()
        .await;
    assert_eq!(result, vec![2, 3]);
}

#[tokio::test]
async fn skip_and_map() {
    // Test skip combined with map
    let result: Vec<String> = stream::iter(vec![1, 2, 3, 4, 5])
        .skip(2) // [3, 4, 5]
        .map(|x| format!("item_{x}"))
        .collect()
        .await;
    assert_eq!(result, vec!["item_3", "item_4", "item_5"]);
}

#[tokio::test]
async fn skip_very_large_number() {
    // Test with very large skip value (edge case for usize)
    let result: Vec<i32> = stream::iter(vec![1, 2, 3]).skip(usize::MAX).collect().await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn skip_and_size_hint_complex() {
    // Test size hint calculations in complex scenarios
    let stream1 = stream::iter(0..10).skip(3); // Should be 7 elements
    let (lower1, upper1) = stream1.size_hint();
    assert_eq!(lower1, 7);
    assert_eq!(upper1, Some(7));

    let stream2 = stream::iter(0..5).skip(10); // Should be 0 elements
    let (lower2, upper2) = stream2.size_hint();
    assert_eq!(lower2, 0);
    assert_eq!(upper2, Some(0));
}

#[tokio::test]
async fn skip_consistent_with_iterator() {
    // Verify skip behaves consistently with std::iter::Iterator::skip
    let vec_data = vec![10, 20, 30, 40, 50];

    let iterator_result: Vec<i32> = vec_data.iter().copied().skip(2).collect();
    let stream_result: Vec<i32> = stream::iter(vec_data).skip(2).collect().await;

    assert_eq!(iterator_result, stream_result);
}
