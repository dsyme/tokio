#![warn(rust_2018_idioms)]
#![cfg(all(feature = "time", feature = "sync", feature = "io-util"))]

use std::task::Poll;
use tokio::time::{self, interval, sleep, Duration};
use tokio_stream::{Stream, StreamExt};
use tokio_test::*;

use futures::stream;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[tokio::test]
async fn empty_stream() {
    time::pause();

    let stream = stream::empty::<i32>().timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn immediate_items() {
    time::pause();

    let stream = stream::iter(vec![1, 2, 3]).timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    assert_ready_eq!(stream.poll_next(), Some(Ok(1)));
    assert_ready_eq!(stream.poll_next(), Some(Ok(2)));
    assert_ready_eq!(stream.poll_next(), Some(Ok(3)));
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn single_item_stream() {
    time::pause();

    let stream = stream::iter(vec![42]).timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    assert_ready_eq!(stream.poll_next(), Some(Ok(42)));
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn timeout_behavior_basic() {
    time::pause();

    // Item that takes 200ms with 100ms timeout interval should generate timeouts
    let stream = stream::iter(vec![1])
        .then(|x| async move {
            sleep(ms(200)).await;
            x
        })
        .timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    // Start processing
    assert_pending!(stream.poll_next());

    // Should get timeout at some point
    time::advance(ms(100)).await;
    let result = assert_ready!(stream.poll_next());
    assert!(result.unwrap().is_err());

    // Eventually should get the item
    time::advance(ms(150)).await; // Advance enough to complete the item
    loop {
        match stream.poll_next() {
            Poll::Ready(Some(Ok(1))) => break, // Got the item!
            Poll::Ready(Some(Ok(_))) => panic!("Got unexpected item"),
            Poll::Ready(Some(Err(_))) => continue, // Another timeout
            Poll::Ready(None) => panic!("Stream ended without item"),
            Poll::Pending => {
                time::advance(ms(10)).await;
            }
        }
    }

    // Stream should end
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn multiple_slow_items() {
    time::pause();

    let stream = stream::iter(vec![1, 2])
        .then(|x| async move {
            sleep(ms(150)).await; // Longer than timeout interval
            x
        })
        .timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    let mut items_received = Vec::new();
    let mut errors_received = 0;

    // Process stream until completion with more robust timing
    for _ in 0..50 {
        // Give plenty of iterations
        time::advance(ms(25)).await;

        match stream.poll_next() {
            Poll::Ready(Some(Ok(item))) => items_received.push(item),
            Poll::Ready(Some(Err(_))) => errors_received += 1,
            Poll::Ready(None) => break,
            Poll::Pending => continue,
        }

        // If we have both items, we can break early
        if items_received.len() == 2 {
            // Let it run a bit more to get to the end
            for _ in 0..5 {
                time::advance(ms(25)).await;
                if let Poll::Ready(None) = stream.poll_next() {
                    break;
                }
            }
            break;
        }
    }

    // Should have received both items and some timeout errors
    assert_eq!(items_received, vec![1, 2]);
    assert!(
        errors_received > 0,
        "Should have received some timeout errors"
    );
}

#[tokio::test]
async fn size_hint_behavior() {
    let stream = stream::iter(vec![1, 2, 3]).timeout_repeating(interval(ms(100)));

    let (lower, upper) = stream.size_hint();
    assert_eq!(lower, 3); // From underlying stream
    assert_eq!(upper, None); // Can insert infinite timeout errors
}

#[tokio::test]
async fn debug_formatting() {
    let stream = stream::iter(vec![1, 2, 3]).timeout_repeating(interval(ms(100)));

    let debug_str = format!("{stream:?}");
    assert!(debug_str.contains("TimeoutRepeating"));
}

#[tokio::test]
async fn mixed_fast_and_slow_items() {
    time::pause();

    // First item fast, second slow
    let items = vec![(1, 50), (2, 150)]; // (value, delay_ms)
    let stream = stream::iter(items)
        .then(|(value, delay)| async move {
            sleep(ms(delay)).await;
            value
        })
        .timeout_repeating(interval(ms(100)));
    let mut stream = task::spawn(stream);

    // First item should be ready eventually without timeout (allow for timing variations)
    assert_pending!(stream.poll_next());

    let mut got_first_item = false;
    for _ in 0..10 {
        time::advance(ms(25)).await;
        match stream.poll_next() {
            Poll::Ready(Some(Ok(1))) => {
                got_first_item = true;
                break;
            }
            Poll::Ready(Some(Err(_))) => {
                // Unexpected timeout, but continue
                continue;
            }
            Poll::Ready(other) => panic!("Unexpected result: {other:?}"),
            Poll::Pending => continue,
        }
    }
    assert!(got_first_item, "Should have received first item");

    // Second item should timeout and then complete
    assert_pending!(stream.poll_next());

    let mut got_timeout = false;
    let mut got_item = false;

    for _ in 0..20 {
        // Give it many chances
        time::advance(ms(25)).await;
        match stream.poll_next() {
            Poll::Ready(Some(Ok(2))) => {
                got_item = true;
                break;
            }
            Poll::Ready(Some(Ok(_))) => panic!("Got unexpected item"),
            Poll::Ready(Some(Err(_))) => {
                got_timeout = true;
            }
            Poll::Ready(None) => panic!("Stream ended unexpectedly"),
            Poll::Pending => continue,
        }
    }

    assert!(got_timeout, "Should have received at least one timeout");
    assert!(got_item, "Should have eventually received the item");

    // Stream should end
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn chaining_with_filter_map() {
    time::pause();

    // Chain with filter_map to remove timeout errors
    let stream = stream::iter(vec![1, 2, 3])
        .then(|x| async move {
            if x == 2 {
                sleep(ms(150)).await; // This will timeout
            } else {
                sleep(ms(50)).await; // These won't
            }
            x
        })
        .timeout_repeating(interval(ms(100)))
        .filter_map(|result| result.ok());
    let mut stream = task::spawn(stream);

    // Should eventually get all three items, filtering out timeouts
    let mut items = Vec::new();
    for _ in 0..100 {
        // Give it many chances
        time::advance(ms(10)).await;
        match stream.poll_next() {
            Poll::Ready(Some(item)) => items.push(item),
            Poll::Ready(None) => break,
            Poll::Pending => continue,
        }
        if items.len() == 3 {
            break;
        }
    }

    assert_eq!(items, vec![1, 2, 3]);
    assert_ready_eq!(stream.poll_next(), None);
}

#[tokio::test]
async fn timeout_with_pending_stream() {
    time::pause();

    // Stream that never produces items
    let stream = stream::pending::<i32>().timeout_repeating(interval(ms(50)));
    let mut stream = task::spawn(stream);

    assert_pending!(stream.poll_next());

    // Should get timeout errors repeatedly
    let mut timeout_count = 0;
    for _ in 0..5 {
        time::advance(ms(50)).await;
        let result = assert_ready!(stream.poll_next());
        assert!(result.unwrap().is_err());
        timeout_count += 1;

        assert_pending!(stream.poll_next());
    }

    assert_eq!(timeout_count, 5);
}
