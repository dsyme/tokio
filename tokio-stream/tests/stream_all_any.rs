use tokio_stream::{self as stream, StreamExt};
use tokio_test::{assert_pending, assert_ready, task};

#[tokio::test]
async fn all_empty_stream() {
    // An empty stream should return true for `all`
    let result = stream::empty::<i32>().all(|_| false).await;
    assert!(result);
}

#[tokio::test]
async fn all_single_true() {
    // Single element stream where predicate returns true
    let result = stream::iter(vec![42]).all(|x| x > 0).await;
    assert!(result);
}

#[tokio::test]
async fn all_single_false() {
    // Single element stream where predicate returns false
    let result = stream::iter(vec![42]).all(|x| x < 0).await;
    assert!(!result);
}

#[tokio::test]
async fn all_multiple_all_true() {
    // Multiple elements, all satisfy predicate
    let result = stream::iter(vec![1, 2, 3, 4, 5]).all(|x| x > 0).await;
    assert!(result);
}

#[tokio::test]
async fn all_multiple_some_false() {
    // Multiple elements, one fails predicate (should short-circuit)
    let result = stream::iter(vec![1, 2, -1, 4, 5]).all(|x| x > 0).await;
    assert!(!result);
}

#[tokio::test]
async fn all_early_termination() {
    // Test that `all` terminates early when predicate fails
    let mut call_count = 0;
    let result = stream::iter(vec![1, 2, -1, 4, 5])
        .all(|x| {
            call_count += 1;
            x > 0
        })
        .await;
    assert!(!result);
    assert_eq!(call_count, 3); // Should stop at the third element (-1)
}

#[tokio::test]
async fn all_large_stream_chunking() {
    // Test the chunking behavior (32 items before yielding)
    let large_vec: Vec<i32> = (1..=100).collect();
    let result = stream::iter(large_vec).all(|x| x > 0).await;
    assert!(result);
}

#[tokio::test]
async fn all_pending_stream() {
    // Test with a stream that can be pending
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        drop(tx); // Close the stream
        
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .all(|x| x > 0)
            .await
    });
    
    assert_ready!(task.poll());
}

#[tokio::test]
async fn any_empty_stream() {
    // An empty stream should return false for `any`
    let result = stream::empty::<i32>().any(|_| true).await;
    assert!(!result);
}

#[tokio::test]
async fn any_single_true() {
    // Single element stream where predicate returns true
    let result = stream::iter(vec![42]).any(|x| x > 0).await;
    assert!(result);
}

#[tokio::test]
async fn any_single_false() {
    // Single element stream where predicate returns false
    let result = stream::iter(vec![-42]).any(|x| x > 0).await;
    assert!(!result);
}

#[tokio::test]
async fn any_multiple_none_true() {
    // Multiple elements, none satisfy predicate
    let result = stream::iter(vec![-1, -2, -3, -4, -5]).any(|x| x > 0).await;
    assert!(!result);
}

#[tokio::test]
async fn any_multiple_some_true() {
    // Multiple elements, one satisfies predicate (should short-circuit)
    let result = stream::iter(vec![-1, -2, 3, 4, 5]).any(|x| x > 0).await;
    assert!(result);
}

#[tokio::test]
async fn any_early_termination() {
    // Test that `any` terminates early when predicate succeeds
    let mut call_count = 0;
    let result = stream::iter(vec![-1, -2, 3, 4, 5])
        .any(|x| {
            call_count += 1;
            x > 0
        })
        .await;
    assert!(result);
    assert_eq!(call_count, 3); // Should stop at the third element (3)
}

#[tokio::test]
async fn any_large_stream_chunking() {
    // Test the chunking behavior (32 items before yielding)
    let large_vec: Vec<i32> = (-100..=0).collect();
    let result = stream::iter(large_vec).any(|x| x == 0).await;
    assert!(result);
}

#[tokio::test]
async fn any_pending_stream() {
    // Test with a stream that can be pending
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Send some items
        tx.send(-1).unwrap();
        tx.send(-2).unwrap();
        tx.send(3).unwrap();
        drop(tx); // Close the stream
        
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .any(|x| x > 0)
            .await
    });
    
    assert_ready!(task.poll());
}

#[tokio::test]
async fn all_any_with_different_types() {
    // Test with string data
    let strings = vec!["hello", "world", "rust"];
    
    // All strings have length > 2
    let all_result = stream::iter(strings.clone()).all(|s| s.len() > 2).await;
    assert!(all_result);
    
    // Any string contains 'o'
    let any_result = stream::iter(strings).any(|s| s.contains('o')).await;
    assert!(any_result);
}

#[tokio::test]
async fn all_any_mutation_in_closure() {
    // Test that the closure can mutate captured variables
    let mut counter = 0;
    let result = stream::iter(vec![1, 2, 3]).all(|x| {
        counter += x;
        x > 0
    }).await;
    assert!(result);
    assert_eq!(counter, 6);
    
    counter = 0;
    let result = stream::iter(vec![1, 2, 3]).any(|x| {
        counter += x;
        x > 2
    }).await;
    assert!(result);
    assert_eq!(counter, 6); // 1 + 2 + 3, all items processed before finding match
}