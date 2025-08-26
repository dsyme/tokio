use tokio::sync::broadcast;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tokio_stream::StreamExt;
use tokio_test::{assert_pending, assert_ready, task};

#[tokio::test]
async fn basic_broadcast_stream() {
    let (tx, rx) = broadcast::channel::<i32>(16);

    tx.send(1).unwrap();
    tx.send(2).unwrap();

    let mut stream = BroadcastStream::new(rx);

    assert_eq!(stream.next().await, Some(Ok(1)));
    assert_eq!(stream.next().await, Some(Ok(2)));

    // Send more items
    tx.send(3).unwrap();
    tx.send(4).unwrap();

    assert_eq!(stream.next().await, Some(Ok(3)));
    assert_eq!(stream.next().await, Some(Ok(4)));

    // Drop sender to close the stream
    drop(tx);
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn empty_broadcast_stream() {
    let (_tx, rx) = broadcast::channel::<i32>(16);
    let mut stream = BroadcastStream::new(rx);

    // Stream should be pending since no items sent yet
    let mut task = task::spawn(stream.next());
    assert_pending!(task.poll());
}

#[tokio::test]
async fn closed_broadcast_stream() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let mut stream = BroadcastStream::new(rx);

    // Drop sender immediately to close the channel
    drop(tx);

    // Stream should return None for closed channel
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn lagged_broadcast_stream() {
    let (tx, rx) = broadcast::channel::<i32>(2); // Small capacity to trigger lagging
    let mut stream = BroadcastStream::new(rx);

    // Fill the channel and send more to cause lagging
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap(); // This should cause receiver to lag

    // First call might get a message or a lag error, depending on timing
    let first_result = stream.next().await;
    match first_result {
        Some(Ok(value)) => {
            // If we got a value, it should be the oldest message still in the channel
            println!("Got value: {value}");
            assert!(value >= 2); // Should be 2 or 3
        }
        Some(Err(BroadcastStreamRecvError::Lagged(n))) => {
            println!("Got lag error: {n} messages skipped");
            assert!(n >= 1); // Should have lagged by at least 1 message
                             // After lag error, next call should get the oldest available message
            let next_result = stream.next().await;
            match next_result {
                Some(Ok(value)) => {
                    println!("Got value after lag: {value}");
                    assert!(value >= 2); // Should be 2 or 3
                }
                other => panic!("Expected Ok value after lag, got: {other:?}"),
            }
        }
        None => panic!("Expected Some result, got None"),
    }
}

#[tokio::test]
async fn multiple_senders_broadcast_stream() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let tx2 = tx.clone();

    let mut stream = BroadcastStream::new(rx);

    tx.send(1).unwrap();
    tx2.send(2).unwrap();
    tx.send(3).unwrap();

    assert_eq!(stream.next().await, Some(Ok(1)));
    assert_eq!(stream.next().await, Some(Ok(2)));
    assert_eq!(stream.next().await, Some(Ok(3)));

    drop(tx);
    drop(tx2);
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn from_trait_implementation() {
    let (_tx, rx) = broadcast::channel::<i32>(16);

    // Test the From trait implementation
    let stream: BroadcastStream<i32> = rx.into();

    // Verify it creates a valid stream
    let debug_str = format!("{stream:?}");
    assert_eq!(debug_str, "BroadcastStream");
}

#[tokio::test]
async fn debug_formatting() {
    let (_tx, rx) = broadcast::channel::<i32>(16);
    let stream = BroadcastStream::new(rx);

    let debug_str = format!("{stream:?}");
    assert_eq!(debug_str, "BroadcastStream");
}

#[tokio::test]
async fn error_display_formatting() {
    use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

    let error = BroadcastStreamRecvError::Lagged(5);
    let display_str = format!("{error}");
    assert_eq!(display_str, "channel lagged by 5");

    let error2 = BroadcastStreamRecvError::Lagged(1);
    let display_str2 = format!("{error2}");
    assert_eq!(display_str2, "channel lagged by 1");
}

#[tokio::test]
async fn error_clone_and_eq() {
    use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

    let error1 = BroadcastStreamRecvError::Lagged(10);
    let error2 = error1.clone();
    let error3 = BroadcastStreamRecvError::Lagged(20);

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[tokio::test]
async fn error_as_std_error() {
    use std::error::Error;
    use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

    let error = BroadcastStreamRecvError::Lagged(5);
    let _error_trait: &dyn Error = &error;

    // Verify it implements std::error::Error
    assert!(error.source().is_none());
}

#[tokio::test]
async fn multiple_receivers_same_broadcast() {
    let (tx, rx1) = broadcast::channel::<String>(16);
    let rx2 = tx.subscribe();

    let mut stream1 = BroadcastStream::new(rx1);
    let mut stream2 = BroadcastStream::new(rx2);

    tx.send("hello".to_string()).unwrap();
    tx.send("world".to_string()).unwrap();

    // Both streams should receive the same messages
    assert_eq!(stream1.next().await, Some(Ok("hello".to_string())));
    assert_eq!(stream2.next().await, Some(Ok("hello".to_string())));

    assert_eq!(stream1.next().await, Some(Ok("world".to_string())));
    assert_eq!(stream2.next().await, Some(Ok("world".to_string())));

    drop(tx);
    assert_eq!(stream1.next().await, None);
    assert_eq!(stream2.next().await, None);
}

#[tokio::test]
async fn string_type_broadcast() {
    let (tx, rx) = broadcast::channel::<String>(16);
    let mut stream = BroadcastStream::new(rx);

    tx.send("test".to_string()).unwrap();
    tx.send("message".to_string()).unwrap();

    assert_eq!(stream.next().await, Some(Ok("test".to_string())));
    assert_eq!(stream.next().await, Some(Ok("message".to_string())));

    drop(tx);
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn complex_type_broadcast() {
    #[derive(Debug, Clone, PartialEq)]
    struct Data {
        id: u32,
        name: String,
    }

    let (tx, rx) = broadcast::channel::<Data>(16);
    let mut stream = BroadcastStream::new(rx);

    let data1 = Data {
        id: 1,
        name: "first".to_string(),
    };
    let data2 = Data {
        id: 2,
        name: "second".to_string(),
    };

    tx.send(data1.clone()).unwrap();
    tx.send(data2.clone()).unwrap();

    assert_eq!(stream.next().await, Some(Ok(data1)));
    assert_eq!(stream.next().await, Some(Ok(data2)));

    drop(tx);
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn collect_broadcast_stream() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let stream = BroadcastStream::new(rx);

    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    drop(tx); // Close the channel

    let results: Vec<Result<i32, _>> = stream.collect().await;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], Ok(1));
    assert_eq!(results[1], Ok(2));
    assert_eq!(results[2], Ok(3));
}

#[tokio::test]
async fn stream_chaining_with_broadcast() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let stream = BroadcastStream::new(rx);

    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    tx.send(4).unwrap();
    drop(tx);

    let results: Vec<i32> = stream
        .filter_map(|result| result.ok())
        .filter(|&x| x % 2 == 0)
        .collect()
        .await;

    assert_eq!(results, vec![2, 4]);
}

#[tokio::test]
async fn large_message_count() {
    let (tx, rx) = broadcast::channel::<usize>(1000);
    let mut stream = BroadcastStream::new(rx);

    // Send many messages
    for i in 0..500 {
        tx.send(i).unwrap();
    }
    drop(tx);

    // Collect all messages
    let mut received = Vec::new();
    while let Some(result) = stream.next().await {
        received.push(result.unwrap());
    }

    assert_eq!(received.len(), 500);
    for (i, &value) in received.iter().enumerate() {
        assert_eq!(value, i);
    }
}

#[tokio::test]
async fn interleaved_send_receive() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let mut stream = BroadcastStream::new(rx);

    // Interleave sends and receives
    tx.send(1).unwrap();
    assert_eq!(stream.next().await, Some(Ok(1)));

    tx.send(2).unwrap();
    tx.send(3).unwrap();
    assert_eq!(stream.next().await, Some(Ok(2)));
    assert_eq!(stream.next().await, Some(Ok(3)));

    tx.send(4).unwrap();
    assert_eq!(stream.next().await, Some(Ok(4)));

    drop(tx);
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn pending_stream_behavior() {
    let (tx, rx) = broadcast::channel::<i32>(16);
    let mut stream = BroadcastStream::new(rx);

    // Stream should be pending when no messages are available
    let mut task = task::spawn(stream.next());
    assert_pending!(task.poll());

    // Send a message, then task should be ready
    tx.send(42).unwrap();
    let result = assert_ready!(task.poll());
    assert_eq!(result, Some(Ok(42)));
}
