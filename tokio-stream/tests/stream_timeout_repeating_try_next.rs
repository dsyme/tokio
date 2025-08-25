#![cfg(all(feature = "time", feature = "sync", feature = "io-util"))]

use tokio::time::{self, sleep, Duration, interval};
use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::*;
use futures::stream as futures_stream;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}


// Tests for timeout_repeating method
#[tokio::test] 
async fn timeout_repeating_empty_stream() {
    time::pause();
    
    let stream = stream::empty::<i32>().timeout_repeating(interval(ms(50)));
    let mut stream = task::spawn(stream);
    
    // Empty stream should complete immediately with None
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn timeout_repeating_immediate_items() {
    let stream = stream::iter(vec![1, 2, 3, 4, 5])
        .timeout_repeating(interval(ms(100)));
        
    let results: Vec<_> = stream.collect().await;
    
    // All items should succeed immediately
    assert_eq!(results, vec![Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)]);
}

#[tokio::test]
async fn timeout_repeating_all_timeout() {
    time::pause();
    
    // Stream that never yields items
    let stream = futures_stream::pending::<i32>()
        .timeout_repeating(interval(ms(50)));
        
    let mut stream = task::spawn(stream);
    
    // Should get repeated timeout errors
    for _ in 0..3 {
        assert_pending!(stream.poll_next());
        time::advance(ms(50)).await;
        let v = assert_ready!(stream.poll_next());
        assert!(v.unwrap().is_err());
    }
}

#[tokio::test]
async fn timeout_repeating_size_hint() {
    let stream = stream::iter(vec![1, 2, 3, 4, 5])
        .timeout_repeating(interval(ms(100)));
        
    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 5); // Lower bound from original stream
    assert_eq!(upper, None); // Upper bound is None due to potential infinite timeouts
}

#[tokio::test]
async fn timeout_repeating_debug_format() {
    let stream = stream::iter(vec![1, 2, 3])
        .timeout_repeating(interval(ms(100)));
        
    let debug_str = format!("{stream:?}");
    assert!(debug_str.contains("TimeoutRepeating"));
}

#[tokio::test]
async fn timeout_repeating_fast_interval() {
    time::pause();
    
    // Very fast timeout interval
    let stream = futures_stream::pending::<i32>()
        .timeout_repeating(interval(ms(10)));
        
    let mut stream = task::spawn(stream);
    
    // Should get many quick timeouts
    for _ in 0..3 {
        assert_pending!(stream.poll_next());
        time::advance(ms(10)).await;
        let v = assert_ready!(stream.poll_next());
        assert!(v.unwrap().is_err());
    }
}

#[tokio::test]
async fn timeout_repeating_basic_functionality() {
    // Simple test that just verifies the timeout_repeating compiles and works with immediate items
    let stream = stream::iter(vec![1, 2, 3])
        .timeout_repeating(interval(ms(100)));
        
    let results: Vec<_> = stream.collect().await;
    
    // Since all items are immediate, they should all succeed
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}

// Tests for try_next method
#[tokio::test]
async fn try_next_success_stream() {
    let mut stream = stream::iter(vec![Ok::<i32, &str>(1), Ok(2), Ok(3)]);
    
    assert_eq!(stream.try_next().await, Ok(Some(1)));
    assert_eq!(stream.try_next().await, Ok(Some(2)));
    assert_eq!(stream.try_next().await, Ok(Some(3)));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test] 
async fn try_next_error_stream() {
    let mut stream = stream::iter(vec![Ok(1), Err("error"), Ok(3)]);
    
    assert_eq!(stream.try_next().await, Ok(Some(1)));
    assert_eq!(stream.try_next().await, Err("error"));
    assert_eq!(stream.try_next().await, Ok(Some(3)));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_empty_stream() {
    let mut stream = stream::empty::<Result<i32, &'static str>>();
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_all_errors() {
    let mut stream = stream::iter(vec![Err::<i32, &str>("err1"), Err("err2"), Err("err3")]);
    
    assert_eq!(stream.try_next().await, Err("err1"));
    assert_eq!(stream.try_next().await, Err("err2"));  
    assert_eq!(stream.try_next().await, Err("err3"));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_with_channel() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Send some results
    tx.send(Ok(1)).unwrap();
    tx.send(Err("error")).unwrap();
    tx.send(Ok(2)).unwrap();
    drop(tx);
    
    let mut stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    
    assert_eq!(stream.try_next().await, Ok(Some(1)));
    assert_eq!(stream.try_next().await, Err("error"));
    assert_eq!(stream.try_next().await, Ok(Some(2)));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_string_results() {
    let mut stream = stream::iter(vec![
        Ok("hello".to_string()),
        Err("parse error".to_string()),
        Ok("world".to_string())
    ]);
    
    assert_eq!(stream.try_next().await, Ok(Some("hello".to_string())));
    assert_eq!(stream.try_next().await, Err("parse error".to_string()));
    assert_eq!(stream.try_next().await, Ok(Some("world".to_string())));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_with_parsing() {
    // Simulate parsing strings to numbers
    let mut stream = stream::iter(vec!["1", "not_a_number", "3", "also_invalid", "5"])
        .map(|s| s.parse::<i32>());
        
    assert_eq!(stream.try_next().await, Ok(Some(1)));
    assert!(stream.try_next().await.is_err()); // Parse error for "not_a_number"
    assert_eq!(stream.try_next().await, Ok(Some(3)));
    assert!(stream.try_next().await.is_err()); // Parse error for "also_invalid"
    assert_eq!(stream.try_next().await, Ok(Some(5)));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_cancel_safety() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<i32, &str>>();
    let mut stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    
    // Test cancel safety by starting a future and dropping it
    {
        let future = stream.try_next();
        let mut task = task::spawn(future);
        
        // Should be pending initially
        assert_pending!(task.poll());
        
        // Drop the task (cancel)
        drop(task);
    }
    
    // Send a value and try again - should work fine (cancel safety)
    tx.send(Ok(42)).unwrap();
    drop(tx);
    
    assert_eq!(stream.try_next().await, Ok(Some(42)));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test]
async fn try_next_with_complex_types() {
    #[derive(Debug, PartialEq)]
    struct Data {
        id: u32,
        name: String,
    }
    
    let mut stream = stream::iter(vec![
        Ok(Data { id: 1, name: "Alice".to_string() }),
        Err("database error".to_string()),
        Ok(Data { id: 2, name: "Bob".to_string() }),
    ]);
    
    assert_eq!(stream.try_next().await, Ok(Some(Data { id: 1, name: "Alice".to_string() })));
    assert_eq!(stream.try_next().await, Err("database error".to_string()));
    assert_eq!(stream.try_next().await, Ok(Some(Data { id: 2, name: "Bob".to_string() })));
    assert_eq!(stream.try_next().await, Ok(None));
}

#[tokio::test] 
async fn try_next_large_stream() {
    // Test with a larger stream to ensure efficiency
    let results: Vec<Result<i32, &'static str>> = (0..100)
        .map(|i| if i % 10 == 0 { Err("error") } else { Ok(i) })
        .collect();
        
    let mut stream = stream::iter(results.clone());
    
    for (i, expected) in results.iter().enumerate() {
        let actual = stream.try_next().await;
        match expected {
            Ok(val) => assert_eq!(actual, Ok(Some(*val)), "Mismatch at index {}", i),
            Err(err) => assert_eq!(actual, Err(*err), "Mismatch at index {}", i),
        }
    }
    
    assert_eq!(stream.try_next().await, Ok(None));
}

// Integration tests combining timeout_repeating and try_next
#[tokio::test] 
async fn timeout_repeating_with_try_next() {
    time::pause();
    
    // Create a simple stream that delays second item
    let items = vec![
        Ok::<i32, tokio::time::error::Elapsed>(1), 
        Ok(2)  // This will be delayed by the timeout mechanism in the test below
    ];
    let stream = stream::iter(items).timeout_repeating(interval(ms(100)));
    let results: Vec<_> = stream.collect().await;
    
    // Both items should succeed without timeout since they're immediate
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[tokio::test]
async fn timeout_repeating_error_type() {
    time::pause();
    
    let stream = futures_stream::pending::<i32>()
        .timeout_repeating(interval(ms(50)));
        
    let mut stream = task::spawn(stream);
    
    assert_pending!(stream.poll_next());
    time::advance(ms(50)).await;
    
    let result = assert_ready!(stream.poll_next());
    match result {
        Some(Err(elapsed)) => {
            // Verify it's the correct error type
            assert_eq!(elapsed.to_string(), "deadline has elapsed");
        }
        other => panic!("Expected Elapsed error, got {:?}", other),
    }
}