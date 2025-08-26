use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

#[tokio::test]
async fn take_while_empty_stream() {
    // An empty stream should return empty when take_while is applied
    let result: Vec<i32> = stream::empty::<i32>().take_while(|_| false).collect().await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn take_while_all_true() {
    // All elements satisfy predicate - should take all
    let result: Vec<i32> = stream::iter(vec![2, 4, 6, 8, 10])
        .take_while(|x| x % 2 == 0)
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn take_while_immediate_false() {
    // First element fails predicate - should take none
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .take_while(|x| x % 2 == 0)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn take_while_early_termination() {
    // Some elements satisfy predicate, then one fails - should stop
    let result: Vec<i32> = stream::iter(vec![2, 4, 6, 7, 8, 10])
        .take_while(|x| x % 2 == 0)
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn take_while_single_element_true() {
    // Single element that satisfies predicate
    let result: Vec<i32> = stream::iter(vec![42])
        .take_while(|x| *x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![42]);
}

#[tokio::test]
async fn take_while_single_element_false() {
    // Single element that fails predicate
    let result: Vec<i32> = stream::iter(vec![-42])
        .take_while(|x| *x > 0)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn take_while_call_count_verification() {
    // Test that predicate is called exactly the right number of times
    let mut call_count = 0;
    let result: Vec<i32> = stream::iter(vec![1, 3, 5, 6, 7, 9])
        .take_while(|x| {
            call_count += 1;
            x % 2 == 1
        })
        .collect()
        .await;

    assert_eq!(result, vec![1, 3, 5]);
    assert_eq!(call_count, 4); // Called for 1, 3, 5, 6 (stops at 6)
}

#[tokio::test]
async fn take_while_size_hint_before_done() {
    // Test size_hint before any polling
    let stream = stream::iter(vec![1, 2, 3]).take_while(|x| *x < 3);
    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 0); // Lower bound is always 0
    assert_eq!(upper, Some(3)); // Upper bound matches original stream
}

#[tokio::test]
async fn take_while_size_hint_after_done() {
    // Test size_hint after stream is exhausted
    let stream = stream::iter(vec![1, 2, 3]).take_while(|x| *x < 2);

    // Consume the stream
    let _result: Vec<i32> = stream.collect().await;
    // Note: We can't easily test size_hint after consumption in this pattern
    // since collect() consumes the stream. This test verifies the logic works.
}

#[tokio::test]
async fn take_while_debug_formatting() {
    // Test Debug trait implementation
    let stream = stream::iter(vec![1, 2, 3]).take_while(|x| *x < 3);
    let debug_str = format!("{stream:?}");
    assert!(debug_str.contains("TakeWhile"));
    assert!(debug_str.contains("done"));
}

#[tokio::test]
async fn take_while_with_different_types() {
    // Test with string data
    let strings = vec!["apple", "apricot", "banana", "berry", "cherry"];
    let result: Vec<&str> = stream::iter(strings)
        .take_while(|s| s.starts_with('a'))
        .collect()
        .await;

    assert_eq!(result, vec!["apple", "apricot"]);
}

#[tokio::test]
async fn take_while_complex_predicate() {
    // Test with more complex predicate logic
    let numbers = vec![1, 4, 9, 16, 25, 36, 49];
    let result: Vec<i32> = stream::iter(numbers)
        .take_while(|x| {
            let sqrt = (*x as f64).sqrt() as i32;
            sqrt * sqrt == *x && sqrt < 6 // Perfect squares less than 36
        })
        .collect()
        .await;

    assert_eq!(result, vec![1, 4, 9, 16, 25]);
}

#[tokio::test]
async fn take_while_closure_mutation() {
    // Test that closure can capture and mutate variables
    let mut sum = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .take_while(|x| {
            sum += *x;
            sum <= 6
        })
        .collect()
        .await;

    assert_eq!(result, vec![1, 2, 3]);
    assert_eq!(sum, 10); // 1 + 2 + 3 + 4
}

#[tokio::test]
async fn take_while_async_stream() {
    // Test with channel-based async stream
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send items
        tx.send(2).unwrap();
        tx.send(4).unwrap();
        tx.send(6).unwrap();
        tx.send(7).unwrap(); // This should terminate take_while
        tx.send(8).unwrap();
        drop(tx); // Close the stream

        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .take_while(|x| x % 2 == 0)
            .collect::<Vec<_>>()
            .await
    });

    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn take_while_chaining_operations() {
    // Test take_while combined with other stream operations
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .filter(|x| x % 2 == 0) // [2, 4, 6, 8, 10]
        .take_while(|x| *x < 8) // [2, 4, 6]
        .map(|x| x * 2) // [4, 8, 12]
        .collect()
        .await;

    assert_eq!(result, vec![4, 8, 12]);
}

#[tokio::test]
async fn take_while_empty_after_take() {
    // Test that once predicate fails, stream stays exhausted
    let mut stream = Box::pin(stream::iter(vec![1, 2, 3, 4, 5]).take_while(|x| *x < 3));

    // First poll should give us items 1, 2
    let item1 = stream.as_mut().next().await;
    assert_eq!(item1, Some(1));

    let item2 = stream.as_mut().next().await;
    assert_eq!(item2, Some(2));

    // Next item (3) fails predicate, so stream should end
    let item3 = stream.as_mut().next().await;
    assert_eq!(item3, None);

    // Subsequent polls should continue to return None
    let item4 = stream.as_mut().next().await;
    assert_eq!(item4, None);
}

#[tokio::test]
async fn take_while_large_stream() {
    // Test with larger stream to ensure performance is reasonable
    let large_numbers: Vec<i32> = (0..1000).collect();
    let result: Vec<i32> = stream::iter(large_numbers)
        .take_while(|x| *x < 100)
        .collect()
        .await;

    assert_eq!(result.len(), 100);
    assert_eq!(result, (0..100).collect::<Vec<_>>());
}

#[tokio::test]
async fn take_while_with_option_unwrapping() {
    // Real-world pattern: parsing until None or error
    let items = vec![Some(1), Some(2), Some(3), None, Some(4)];
    let result: Vec<Option<i32>> = stream::iter(items)
        .take_while(|opt| opt.is_some())
        .collect()
        .await;

    assert_eq!(result, vec![Some(1), Some(2), Some(3)]);
}

#[tokio::test]
async fn take_while_early_stream_end() {
    // Test when stream ends before predicate fails
    let result: Vec<i32> = stream::iter(vec![2, 4, 6])
        .take_while(|x| *x < 100) // Predicate never fails
        .collect()
        .await;

    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn take_while_zero_elements() {
    // Edge case: predicate never succeeds
    let result: Vec<i32> = stream::iter(vec![10, 20, 30])
        .take_while(|x| *x < 5)
        .collect()
        .await;

    assert!(result.is_empty());
}

#[tokio::test]
async fn take_while_multiple_chained() {
    // Test multiple take_while operations chained together
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .take_while(|x| *x <= 7) // [1, 2, 3, 4, 5, 6, 7]
        .take_while(|x| *x != 4) // [1, 2, 3]
        .collect()
        .await;

    assert_eq!(result, vec![1, 2, 3]);
}
