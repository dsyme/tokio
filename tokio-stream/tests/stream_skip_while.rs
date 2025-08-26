use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

#[tokio::test]
async fn skip_while_empty_stream() {
    // Empty stream should remain empty regardless of predicate
    let result: Vec<i32> = stream::empty::<i32>().skip_while(|_| true).collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn skip_while_single_element_skip() {
    // Single element that should be skipped
    let result: Vec<i32> = stream::iter(vec![1]).skip_while(|x| *x > 0).collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn skip_while_single_element_keep() {
    // Single element that should not be skipped
    let result: Vec<i32> = stream::iter(vec![1]).skip_while(|x| *x < 0).collect().await;
    assert_eq!(result, vec![1]);
}

#[tokio::test]
async fn skip_while_all_skip() {
    // All elements match predicate, so all should be skipped
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .skip_while(|x| *x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn skip_while_none_skip() {
    // No elements match predicate, so none should be skipped
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .skip_while(|x| *x < 0)
        .collect()
        .await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn skip_while_partial_skip() {
    // Some elements match predicate, then one doesn't - key behavior test
    let result: Vec<i32> = stream::iter(vec![1, 2, -1, 4, 5])
        .skip_while(|x| *x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![-1, 4, 5]); // After -1, all remaining elements are kept
}

#[tokio::test]
async fn skip_while_early_termination() {
    // Test that skip_while stops checking predicate after first false
    let mut call_count = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, -1, 4, 5])
        .skip_while(|x| {
            call_count += 1;
            *x > 0
        })
        .collect()
        .await;
    assert_eq!(result, vec![-1, 4, 5]);
    assert_eq!(call_count, 3); // Should stop calling after -1
}

#[tokio::test]
async fn skip_while_size_hints() {
    let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    let skip_while_stream = stream.skip_while(|x| *x <= 2);

    // Before any polling, size hint should be (0, Some(5))
    assert_eq!(skip_while_stream.size_hint(), (0, Some(5)));
}

#[tokio::test]
async fn skip_while_size_hints_after_transition() {
    let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    let mut skip_while_stream = stream.skip_while(|x| *x <= 2);

    // Poll once to get past the skipping phase
    let first = skip_while_stream.next().await;
    assert_eq!(first, Some(3));

    // After transition, size hint should reflect remaining elements
    // This tests the "predicate is None" path in size_hint
    assert_eq!(skip_while_stream.size_hint(), (2, Some(2)));
}

#[tokio::test]
async fn skip_while_debug_formatting() {
    let stream = stream::iter(vec![1, 2, 3]);
    let skip_while_stream = stream.skip_while(|x| *x < 3);

    // Test Debug implementation
    let debug_output = format!("{skip_while_stream:?}");
    assert!(debug_output.contains("SkipWhile"));
    assert!(debug_output.contains("stream"));
}

#[tokio::test]
async fn skip_while_with_strings() {
    // Test with different data types
    let result: Vec<&str> = stream::iter(vec!["a", "bb", "ccc", "dd", "e"])
        .skip_while(|s| s.len() < 3)
        .collect()
        .await;
    assert_eq!(result, vec!["ccc", "dd", "e"]);
}

#[tokio::test]
async fn skip_while_with_complex_predicate() {
    // Test with more complex predicate logic
    let result: Vec<(i32, String)> = stream::iter(vec![
        (1, "small".to_string()),
        (2, "tiny".to_string()),
        (10, "large".to_string()),
        (3, "medium".to_string()),
        (20, "huge".to_string()),
    ])
    .skip_while(|(num, text)| *num < 5 && text.len() < 6)
    .collect()
    .await;

    assert_eq!(
        result,
        vec![
            (10, "large".to_string()),
            (3, "medium".to_string()),
            (20, "huge".to_string()),
        ]
    );
}

#[tokio::test]
async fn skip_while_pending_stream() {
    // Test with a stream that can be pending
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(-1).unwrap();
        tx.send(4).unwrap();
        drop(tx); // Close the stream

        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .skip_while(|x| *x > 0)
            .collect::<Vec<_>>()
            .await
    });

    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![-1, 4]);
}

#[tokio::test]
async fn skip_while_chaining() {
    // Test chaining multiple skip_while operations
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8])
        .skip_while(|x| *x <= 3) // Skip 1, 2, 3
        .skip_while(|x| *x <= 5) // Skip 4, 5 from remaining
        .collect()
        .await;
    assert_eq!(result, vec![6, 7, 8]);
}

#[tokio::test]
async fn skip_while_with_take() {
    // Test combining skip_while with take
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8])
        .skip_while(|x| *x <= 3)
        .take(2)
        .collect()
        .await;
    assert_eq!(result, vec![4, 5]);
}

#[tokio::test]
async fn skip_while_with_filter() {
    // Test combining skip_while with filter
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8])
        .skip_while(|x| *x <= 3)
        .filter(|x| *x % 2 == 0)
        .collect()
        .await;
    assert_eq!(result, vec![4, 6, 8]);
}

#[tokio::test]
async fn skip_while_mutation_in_closure() {
    // Test that the closure can mutate captured variables
    let mut skipped_count = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .skip_while(|x| {
            if *x <= 2 {
                skipped_count += 1;
                true
            } else {
                false
            }
        })
        .collect()
        .await;
    assert_eq!(result, vec![3, 4, 5]);
    assert_eq!(skipped_count, 2); // 1 and 2 were skipped
}

#[tokio::test]
async fn skip_while_zero_elements() {
    // Edge case: skip_while that immediately stops
    let result: Vec<i32> = stream::iter(vec![-1, 1, 2, 3])
        .skip_while(|x| *x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![-1, 1, 2, 3]);
}

#[tokio::test]
async fn skip_while_large_stream() {
    // Test with larger stream to ensure performance is reasonable
    let large_vec: Vec<i32> = (1..=1000).collect();
    let result: Vec<i32> = stream::iter(large_vec)
        .skip_while(|x| *x <= 950)
        .collect()
        .await;

    let expected: Vec<i32> = (951..=1000).collect();
    assert_eq!(result, expected);
    assert_eq!(result.len(), 50);
}

#[tokio::test]
async fn skip_while_option_parsing() {
    // Real-world example: parsing until we hit Some value
    let data = vec![None, None, Some(42), Some(100), None];
    let result: Vec<Option<i32>> = stream::iter(data)
        .skip_while(|opt| opt.is_none())
        .collect()
        .await;
    assert_eq!(result, vec![Some(42), Some(100), None]);
}

#[tokio::test]
async fn skip_while_result_parsing() {
    // Real-world example: skip errors until we get Ok
    let data: Vec<Result<i32, &str>> =
        vec![Err("error1"), Err("error2"), Ok(42), Ok(100), Err("error3")];
    let result: Vec<Result<i32, &str>> = stream::iter(data)
        .skip_while(|res| res.is_err())
        .collect()
        .await;
    assert_eq!(result, vec![Ok(42), Ok(100), Err("error3")]);
}

#[tokio::test]
async fn skip_while_state_consistency() {
    // Test that state is properly maintained between polls
    let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    let mut skip_while_stream = stream.skip_while(|x| *x <= 2);

    // First poll should skip 1, 2 and return 3
    assert_eq!(skip_while_stream.next().await, Some(3));

    // Subsequent polls should not apply predicate
    assert_eq!(skip_while_stream.next().await, Some(4));
    assert_eq!(skip_while_stream.next().await, Some(5));
    assert_eq!(skip_while_stream.next().await, None);
}
