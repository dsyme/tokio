use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

#[tokio::test]
async fn take_zero_from_empty_stream() {
    // Taking 0 elements from an empty stream should return an empty stream
    let result: Vec<i32> = stream::empty::<i32>().take(0).collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn take_some_from_empty_stream() {
    // Taking any positive number from an empty stream should return an empty stream
    let result: Vec<i32> = stream::empty::<i32>().take(5).collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn take_zero_from_non_empty_stream() {
    // Taking 0 elements from any stream should return an empty stream
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).take(0).collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn take_some_from_single_element() {
    // Taking some elements from a single-element stream
    let result: Vec<i32> = stream::iter(vec![42]).take(1).collect().await;
    assert_eq!(result, vec![42]);
    
    let result: Vec<i32> = stream::iter(vec![42]).take(5).collect().await;
    assert_eq!(result, vec![42]);
}

#[tokio::test]
async fn take_partial_from_stream() {
    // Taking fewer elements than available
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).take(3).collect().await;
    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn take_all_from_stream() {
    // Taking exactly all elements
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5]).take(5).collect().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn take_more_than_available() {
    // Taking more elements than available should return all available elements
    let result: Vec<i32> = stream::iter(vec![1, 2, 3]).take(10).collect().await;
    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn take_with_different_types() {
    // Test with string data
    let strings = vec!["hello", "world", "rust", "async"];
    let result: Vec<&str> = stream::iter(strings).take(2).collect().await;
    assert_eq!(result, vec!["hello", "world"]);
}

#[tokio::test]
async fn take_size_hint_zero() {
    // Test size_hint for taking 0 elements
    let stream = stream::iter(vec![1, 2, 3, 4, 5]).take(0);
    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 0);
    assert_eq!(upper, Some(0));
}

#[tokio::test]
async fn take_size_hint_partial() {
    // Test size_hint when taking fewer elements than stream has
    let stream = stream::iter(vec![1, 2, 3, 4, 5]).take(3);
    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

#[tokio::test]
async fn take_size_hint_more_than_stream() {
    // Test size_hint when taking more elements than stream has
    let stream = stream::iter(vec![1, 2, 3]).take(10);
    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 3);
    assert_eq!(upper, Some(3));
}

#[tokio::test]
async fn take_size_hint_unbounded_stream() {
    // Test size_hint with unbounded stream (upper bound None)
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).take(3);
    let (lower, upper) = stream.size_hint();
    // For unbounded streams with items already in the buffer, lower bound reflects items available
    // The upper bound is limited by the take count
    assert!(lower <= 3);
    assert_eq!(upper, Some(3));
    
    drop(tx); // Close to prevent hanging
}

#[tokio::test]
async fn take_debug_formatting() {
    // Test Debug trait implementation
    let stream = stream::iter(vec![1, 2, 3]).take(2);
    let debug_string = format!("{:?}", stream);
    assert!(debug_string.contains("Take"));
}

#[tokio::test]
async fn take_with_pending_stream() {
    // Test with a stream that can be pending
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
            .take(3)
            .collect::<Vec<i32>>()
            .await
    });

    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn take_chaining_operations() {
    // Test chaining multiple take operations
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .take(8)
        .take(5)
        .collect()
        .await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn take_with_other_adaptors() {
    // Test take combined with filter
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .filter(|&x| x % 2 == 0)  // Even numbers: [2, 4, 6, 8, 10]
        .take(3)
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6]);

    // Test take combined with map
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .map(|x| x * 2)
        .take(3)
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn take_large_stream() {
    // Test with a large stream to ensure no performance issues
    let large_vec: Vec<i32> = (1..=1000).collect();
    let result: Vec<i32> = stream::iter(large_vec).take(10).collect().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[tokio::test]
async fn take_complex_data_types() {
    // Test with complex data types (tuples)
    let data = vec![(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")];
    let result: Vec<(i32, &str)> = stream::iter(data).take(3).collect().await;
    assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
}

#[tokio::test]
async fn take_pagination_pattern() {
    // Test a common pagination pattern - skip and take
    let data: Vec<i32> = (1..=20).collect();
    
    // Page 1: first 5 items
    let page1: Vec<i32> = stream::iter(data.clone())
        .take(5)
        .collect()
        .await;
    assert_eq!(page1, vec![1, 2, 3, 4, 5]);
    
    // Page 2: skip 5, take next 5
    let page2: Vec<i32> = stream::iter(data.clone())
        .skip(5)
        .take(5)
        .collect()
        .await;
    assert_eq!(page2, vec![6, 7, 8, 9, 10]);
}

#[tokio::test]
async fn take_edge_case_usize_max() {
    // Test taking usize::MAX elements (edge case)
    let result: Vec<i32> = stream::iter(vec![1, 2, 3])
        .take(usize::MAX)
        .collect()
        .await;
    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn take_early_termination_stream() {
    // Test that take properly handles stream termination
    let mut remaining_calls = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .map(|x| {
            remaining_calls += 1;
            x
        })
        .take(3)
        .collect()
        .await;
    
    assert_eq!(result, vec![1, 2, 3]);
    assert_eq!(remaining_calls, 3); // Should only process 3 items
}

#[tokio::test]
async fn take_remaining_counter_behavior() {
    // Test internal remaining counter behavior through repeated polling
    use futures::stream::StreamExt as FuturesStreamExt;
    
    let mut stream = tokio_stream::StreamExt::take(stream::iter(vec![1, 2, 3, 4, 5]), 3);
    
    // Poll items one by one to test remaining counter
    assert_eq!(FuturesStreamExt::next(&mut stream).await, Some(1));
    assert_eq!(FuturesStreamExt::next(&mut stream).await, Some(2));
    assert_eq!(FuturesStreamExt::next(&mut stream).await, Some(3));
    assert_eq!(FuturesStreamExt::next(&mut stream).await, None);
    assert_eq!(FuturesStreamExt::next(&mut stream).await, None); // Should remain None
}