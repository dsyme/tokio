use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

#[tokio::test]
async fn peekable_empty_stream() {
    // Test peekable with empty stream
    let mut peekable = stream::empty::<i32>().peekable();
    
    // Peek should return None for empty stream
    assert_eq!(peekable.peek().await, None);
    
    // Next should also return None
    assert_eq!(peekable.next().await, None);
    
    // Peek after consuming should still return None
    assert_eq!(peekable.peek().await, None);
}

#[tokio::test]
async fn peekable_single_element() {
    // Test peekable with single element stream
    let mut peekable = stream::iter(vec![42]).peekable();
    
    // Peek should return Some(&42)
    assert_eq!(peekable.peek().await, Some(&42));
    
    // Peek again should return the same value
    assert_eq!(peekable.peek().await, Some(&42));
    
    // Next should return Some(42) and consume the element
    assert_eq!(peekable.next().await, Some(42));
    
    // Peek after consuming should return None
    assert_eq!(peekable.peek().await, None);
    
    // Next after consuming should return None
    assert_eq!(peekable.next().await, None);
}

#[tokio::test]
async fn peekable_multiple_elements() {
    // Test peekable with multiple elements
    let mut peekable = stream::iter(vec![1, 2, 3, 4, 5]).peekable();
    
    // Peek should show the first element
    assert_eq!(peekable.peek().await, Some(&1));
    assert_eq!(peekable.peek().await, Some(&1));
    
    // Consume first element
    assert_eq!(peekable.next().await, Some(1));
    
    // Peek should now show the second element
    assert_eq!(peekable.peek().await, Some(&2));
    
    // Consume second element
    assert_eq!(peekable.next().await, Some(2));
    
    // Continue with remaining elements
    assert_eq!(peekable.next().await, Some(3));
    assert_eq!(peekable.peek().await, Some(&4));
    assert_eq!(peekable.next().await, Some(4));
    assert_eq!(peekable.next().await, Some(5));
    
    // After all elements consumed
    assert_eq!(peekable.peek().await, None);
    assert_eq!(peekable.next().await, None);
}

#[tokio::test]
async fn peekable_without_consuming() {
    // Test collecting all elements without using peek
    let peekable = stream::iter(vec![1, 2, 3, 4, 5]).peekable();
    let result: Vec<i32> = peekable.collect().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn peekable_with_different_types() {
    // Test with string data
    let mut peekable = stream::iter(vec!["hello", "world", "rust"]).peekable();
    
    assert_eq!(peekable.peek().await, Some(&"hello"));
    assert_eq!(peekable.next().await, Some("hello"));
    assert_eq!(peekable.peek().await, Some(&"world"));
    assert_eq!(peekable.next().await, Some("world"));
    assert_eq!(peekable.peek().await, Some(&"rust"));
    assert_eq!(peekable.next().await, Some("rust"));
    assert_eq!(peekable.peek().await, None);
}

#[tokio::test]
async fn peekable_with_pending_stream() {
    // Test with a stream that can be pending
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        drop(tx); // Close the stream

        let mut peekable = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).peekable();
        
        let mut results = Vec::new();
        
        // Peek at first element
        if let Some(val) = peekable.peek().await {
            results.push(*val);
        }
        
        // Consume elements
        while let Some(val) = peekable.next().await {
            results.push(val);
        }
        
        results
    });

    let result = assert_ready!(task.poll());
    // First element appears twice - once from peek, once from next
    assert_eq!(result, vec![1, 1, 2, 3]);
}

#[tokio::test]
async fn peekable_alternating_peek_and_next() {
    // Test alternating peek and next operations
    let mut peekable = stream::iter(vec![10, 20, 30]).peekable();
    
    // Peek, then consume
    assert_eq!(peekable.peek().await, Some(&10));
    assert_eq!(peekable.next().await, Some(10));
    
    // Peek, then consume
    assert_eq!(peekable.peek().await, Some(&20));
    assert_eq!(peekable.next().await, Some(20));
    
    // Peek, then consume
    assert_eq!(peekable.peek().await, Some(&30));
    assert_eq!(peekable.next().await, Some(30));
    
    // Empty
    assert_eq!(peekable.peek().await, None);
    assert_eq!(peekable.next().await, None);
}

#[tokio::test]
async fn peekable_multiple_peeks_before_consume() {
    // Test multiple peek calls before consuming
    let mut peekable = stream::iter(vec![100, 200]).peekable();
    
    // Multiple peeks should return the same value
    assert_eq!(peekable.peek().await, Some(&100));
    assert_eq!(peekable.peek().await, Some(&100));
    assert_eq!(peekable.peek().await, Some(&100));
    
    // Consume the peeked value
    assert_eq!(peekable.next().await, Some(100));
    
    // Peek at next value multiple times
    assert_eq!(peekable.peek().await, Some(&200));
    assert_eq!(peekable.peek().await, Some(&200));
    
    // Consume final value
    assert_eq!(peekable.next().await, Some(200));
    
    // Empty
    assert_eq!(peekable.peek().await, None);
}

#[tokio::test]
async fn peekable_complex_data_types() {
    // Test with complex data types (tuples)
    let data = vec![(1, "a"), (2, "b"), (3, "c")];
    let mut peekable = stream::iter(data).peekable();
    
    assert_eq!(peekable.peek().await, Some(&(1, "a")));
    assert_eq!(peekable.next().await, Some((1, "a")));
    
    assert_eq!(peekable.peek().await, Some(&(2, "b")));
    assert_eq!(peekable.next().await, Some((2, "b")));
    
    assert_eq!(peekable.peek().await, Some(&(3, "c")));
    assert_eq!(peekable.next().await, Some((3, "c")));
    
    assert_eq!(peekable.peek().await, None);
}

#[tokio::test]
async fn peekable_with_stream_adaptors() {
    // Test peekable combined with other stream adaptors
    let mut peekable = stream::iter(vec![1, 2, 3, 4, 5, 6])
        .filter(|&x| x % 2 == 0)  // Even numbers: [2, 4, 6]
        .peekable();
    
    assert_eq!(peekable.peek().await, Some(&2));
    assert_eq!(peekable.next().await, Some(2));
    
    assert_eq!(peekable.peek().await, Some(&4));
    assert_eq!(peekable.next().await, Some(4));
    
    assert_eq!(peekable.peek().await, Some(&6));
    assert_eq!(peekable.next().await, Some(6));
    
    assert_eq!(peekable.peek().await, None);
}

#[tokio::test]
async fn peekable_size_hint() {
    // Test size_hint behavior - peekable wraps a fused stream
    let peekable = stream::iter(vec![1, 2, 3, 4, 5]).peekable();
    let (lower, upper) = peekable.size_hint();
    // Peekable should delegate to the underlying stream's size_hint
    // The exact values depend on the fused stream implementation
    // We just verify the bounds are reasonable
    assert!(lower <= 5);
    if let Some(u) = upper {
        assert!(u >= lower);
    }
    
    // Empty stream
    let peekable = stream::empty::<i32>().peekable();
    let (lower, upper) = peekable.size_hint();
    assert_eq!(lower, 0);
    // Upper bound for empty stream should be Some(0) or None
    if let Some(u) = upper {
        assert_eq!(u, 0);
    }
}

#[tokio::test]
async fn peekable_fused_behavior() {
    // Test that peekable properly handles fused streams (always returns None after first None)
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tx.send(1).unwrap();
    drop(tx); // Close immediately
    
    let mut peekable = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).peekable();
    
    // Should get the one item
    assert_eq!(peekable.next().await, Some(1));
    
    // After stream is exhausted, should consistently return None
    assert_eq!(peekable.peek().await, None);
    assert_eq!(peekable.next().await, None);
    assert_eq!(peekable.peek().await, None);
    assert_eq!(peekable.next().await, None);
}

#[tokio::test]
async fn peekable_collect_after_peek() {
    // Test collecting remaining elements after peeking
    let mut peekable = stream::iter(vec![1, 2, 3, 4, 5]).peekable();
    
    // Peek at first element
    assert_eq!(peekable.peek().await, Some(&1));
    
    // Collect all elements (including the peeked one)
    let result: Vec<i32> = peekable.collect().await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn peekable_partial_consume_then_collect() {
    // Test consuming some elements, then collecting the rest
    let mut peekable = stream::iter(vec![10, 20, 30, 40, 50]).peekable();
    
    // Peek and consume first two elements
    assert_eq!(peekable.peek().await, Some(&10));
    assert_eq!(peekable.next().await, Some(10));
    assert_eq!(peekable.next().await, Some(20));
    
    // Peek at next element
    assert_eq!(peekable.peek().await, Some(&30));
    
    // Collect remaining elements
    let result: Vec<i32> = peekable.collect().await;
    assert_eq!(result, vec![30, 40, 50]);
}

#[tokio::test]
async fn peekable_peek_ownership() {
    // Test that peek returns a reference, not owned value
    let mut peekable = stream::iter(vec![String::from("hello")]).peekable();
    
    // Peek should return &String
    let peeked = peekable.peek().await;
    assert_eq!(peeked, Some(&String::from("hello")));
    
    // We can peek multiple times
    let peeked_again = peekable.peek().await;
    assert_eq!(peeked_again, Some(&String::from("hello")));
    
    // Next should return owned String
    let owned = peekable.next().await;
    assert_eq!(owned, Some(String::from("hello")));
}

#[tokio::test]
async fn peekable_large_stream() {
    // Test with a larger stream to ensure no performance issues
    let large_vec: Vec<i32> = (1..=100).collect();
    let mut peekable = stream::iter(large_vec).peekable();
    
    // Peek at first element
    assert_eq!(peekable.peek().await, Some(&1));
    
    // Consume first 10 elements
    for i in 1..=10 {
        assert_eq!(peekable.next().await, Some(i));
    }
    
    // Peek at 11th element
    assert_eq!(peekable.peek().await, Some(&11));
    
    // Collect remaining elements
    let remaining: Vec<i32> = peekable.collect().await;
    let expected: Vec<i32> = (11..=100).collect();
    assert_eq!(remaining, expected);
}

#[tokio::test]
async fn peekable_edge_case_peek_only() {
    // Test edge case where we only peek and never consume
    let mut peekable = stream::iter(vec![42]).peekable();
    
    // Only peek, never call next
    assert_eq!(peekable.peek().await, Some(&42));
    assert_eq!(peekable.peek().await, Some(&42));
    assert_eq!(peekable.peek().await, Some(&42));
    
    // The element should still be available for consumption
    assert_eq!(peekable.next().await, Some(42));
    assert_eq!(peekable.peek().await, None);
}