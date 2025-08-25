use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

// Tests for map_while method
#[tokio::test]
async fn map_while_empty_stream() {
    let result: Vec<i32> = stream::empty::<i32>()
        .map_while(|x| if x > 0 { Some(x * 2) } else { None })
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn map_while_all_some() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .map_while(|x| Some(x * 2))
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn map_while_immediate_none() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .map_while(|_x| None::<i32>)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn map_while_stops_at_first_none() {
    // This is the key behavior of map_while - it stops at the first None
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, -1, 5, 6])
        .map_while(|x| if x > 0 { Some(x * 2) } else { None })
        .collect()
        .await;
    // Should stop at -1 (first None), so only processes 1, 2, 3
    assert_eq!(result, vec![2, 4, 6]);
}

#[tokio::test]
async fn map_while_single_element_some() {
    let result: Vec<String> = stream::iter(vec![42])
        .map_while(|x| Some(format!("num_{x}")))
        .collect()
        .await;
    assert_eq!(result, vec!["num_42"]);
}

#[tokio::test]
async fn map_while_single_element_none() {
    let result: Vec<String> = stream::iter(vec![42])
        .map_while(|_x| None::<String>)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn map_while_type_conversion() {
    let result: Vec<String> = stream::iter(vec![1, 2, 3, 4, 5])
        .map_while(|x| {
            if x <= 3 {
                Some(format!("item_{x}"))
            } else {
                None
            }
        })
        .collect()
        .await;
    assert_eq!(result, vec!["item_1", "item_2", "item_3"]);
}

#[tokio::test]
async fn map_while_with_strings() {
    let result: Vec<usize> = stream::iter(vec!["hello", "world", "rust", "", "code"])
        .map_while(|s| if !s.is_empty() { Some(s.len()) } else { None })
        .collect()
        .await;
    // Should stop at empty string
    assert_eq!(result, vec![5, 5, 4]);
}

#[tokio::test]
async fn map_while_parsing() {
    // Test parsing numbers from strings, stopping at first invalid
    let result: Vec<i32> = stream::iter(vec!["1", "2", "3", "invalid", "5"])
        .map_while(|s| s.parse::<i32>().ok())
        .collect()
        .await;
    assert_eq!(result, vec![1, 2, 3]); // Stops at "invalid"
}

#[tokio::test]
async fn map_while_with_mutation() {
    let mut counter = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .map_while(|x| {
            counter += 1;
            if x <= 3 {
                Some(x * 10)
            } else {
                None
            }
        })
        .collect()
        .await;
    assert_eq!(result, vec![10, 20, 30]);
    assert_eq!(counter, 4); // Should process 1, 2, 3, then 4 (which returns None)
}

#[tokio::test]
async fn map_while_debug_format() {
    let map_while_stream = stream::iter(vec![1, 2, 3]).map_while(|x| Some(x * 2));
    let debug_str = format!("{map_while_stream:?}");
    assert!(debug_str.contains("MapWhile"));
}

#[tokio::test]
async fn map_while_size_hint() {
    let map_while_stream = stream::iter(vec![1, 2, 3, 4, 5]).map_while(Some);
    let (lower, upper) = map_while_stream.size_hint();
    assert_eq!(lower, 0); // Can't know lower bound due to potential early termination
    assert_eq!(upper, Some(5)); // Upper bound is from original stream
}

#[tokio::test]
async fn map_while_pending_stream() {
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        tx.send(-1).unwrap(); // This should cause map_while to stop
        tx.send(5).unwrap(); // This should not be processed
        drop(tx); // Close the stream

        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .map_while(|x| if x > 0 { Some(x * 100) } else { None })
            .collect::<Vec<_>>()
            .await
    });

    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![100, 200, 300]); // Stops at -1
}

#[tokio::test]
async fn map_while_complex_logic() {
    // Test with more complex transformation logic
    let result: Vec<String> = stream::iter(vec![1, 4, 9, 16, 20, 25])
        .map_while(|x| {
            let sqrt = (x as f64).sqrt();
            if sqrt.fract() == 0.0 {
                // Only perfect squares
                Some(format!("sqrt({x}) = {}", sqrt as i32))
            } else {
                None
            }
        })
        .collect()
        .await;
    // Should stop at 20 (not a perfect square)
    assert_eq!(
        result,
        vec!["sqrt(1) = 1", "sqrt(4) = 2", "sqrt(9) = 3", "sqrt(16) = 4"]
    );
}

#[tokio::test]
async fn map_while_early_termination() {
    // Test that remaining items after None are not processed at all
    let mut processed_count = 0;
    let result: Vec<i32> = stream::iter(vec![2, 4, 6, 7, 8, 10])
        .map_while(|x| {
            processed_count += 1;
            if x % 2 == 0 {
                // Even numbers only
                Some(x / 2)
            } else {
                None
            }
        })
        .collect()
        .await;

    assert_eq!(result, vec![1, 2, 3]); // 2/2, 4/2, 6/2
    assert_eq!(processed_count, 4); // Should process 2, 4, 6, 7 (stops at 7)
}

#[tokio::test]
async fn map_while_chaining() {
    // Test chaining map_while operations
    let result: Vec<String> = stream::iter(vec![1, 2, 3, 4, 5, 6])
        .map_while(|x| if x <= 4 { Some(x) } else { None }) // Take first 4: 1,2,3,4
        .map_while(|x| {
            if x <= 2 {
                Some(format!("num_{x}"))
            } else {
                None
            }
        }) // Take first 2: 1,2
        .collect()
        .await;
    assert_eq!(result, vec!["num_1", "num_2"]);
}

#[tokio::test]
async fn map_while_option_unwrapping() {
    // Test map_while with Option input
    let input = vec![Some(1), Some(2), Some(3), None, Some(5)];
    let result: Vec<i32> = stream::iter(input)
        .map_while(|opt| opt.map(|x| x * 2))
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6]); // Stops at None
}

#[tokio::test]
async fn map_while_with_results() {
    // Test map_while with Result input, converting Ok to Some and Err to None
    let input = vec![Ok(1), Ok(2), Ok(3), Err("error"), Ok(5)];
    let result: Vec<i32> = stream::iter(input)
        .map_while(|res| res.ok().map(|x| x * 3))
        .collect()
        .await;
    assert_eq!(result, vec![3, 6, 9]); // Stops at Err
}
