use tokio_stream::{self as stream, Stream, StreamExt};
use tokio_test::{assert_ready, task};

// Tests for filter method
#[tokio::test]
async fn filter_empty_stream() {
    let result: Vec<i32> = stream::empty::<i32>()
        .filter(|&x| x > 0)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn filter_all_pass() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter(|&x| x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn filter_none_pass() {
    let result: Vec<i32> = stream::iter(vec![-1, -2, -3, -4, -5])
        .filter(|&x| x > 0)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn filter_some_pass() {
    let result: Vec<i32> = stream::iter(vec![1, -2, 3, -4, 5])
        .filter(|&x| x > 0)
        .collect()
        .await;
    assert_eq!(result, vec![1, 3, 5]);
}

#[tokio::test]
async fn filter_with_mutation() {
    let mut counter = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter(|&x| {
            counter += 1;
            x % 2 == 0
        })
        .collect()
        .await;
    assert_eq!(result, vec![2, 4]);
    assert_eq!(counter, 5); // All items should be checked
}

#[tokio::test]
async fn filter_debug_format() {
    let filter_stream = stream::iter(vec![1, 2, 3]).filter(|&x| x > 0);
    let debug_str = format!("{:?}", filter_stream);
    assert!(debug_str.contains("Filter"));
}

#[tokio::test]
async fn filter_size_hint() {
    let filter_stream = stream::iter(vec![1, 2, 3, 4, 5]).filter(|&x| x > 0);
    let (lower, upper) = filter_stream.size_hint();
    assert_eq!(lower, 0); // Can't know lower bound due to filtering
    assert_eq!(upper, Some(5)); // Upper bound is from original stream
}

#[tokio::test]
async fn filter_with_strings() {
    let result: Vec<&str> = stream::iter(vec!["hello", "a", "world", "x", "rust"])
        .filter(|s| s.len() > 2)
        .collect()
        .await;
    assert_eq!(result, vec!["hello", "world", "rust"]);
}

#[tokio::test]
async fn filter_pending_stream() {
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Send some items
        tx.send(1).unwrap();
        tx.send(-2).unwrap();
        tx.send(3).unwrap();
        tx.send(-4).unwrap();
        drop(tx); // Close the stream
        
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .filter(|&x| x > 0)
            .collect::<Vec<_>>()
            .await
    });
    
    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![1, 3]);
}

// Tests for filter_map method
#[tokio::test]
async fn filter_map_empty_stream() {
    let result: Vec<i32> = stream::empty::<i32>()
        .filter_map(|x| if x > 0 { Some(x * 2) } else { None })
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn filter_map_all_some() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter_map(|x| Some(x * 2))
        .collect()
        .await;
    assert_eq!(result, vec![2, 4, 6, 8, 10]);
}

#[tokio::test]
async fn filter_map_all_none() {
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter_map(|_x| None::<i32>)
        .collect()
        .await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn filter_map_mixed() {
    let result: Vec<i32> = stream::iter(vec![1, -2, 3, -4, 5])
        .filter_map(|x| if x > 0 { Some(x * 2) } else { None })
        .collect()
        .await;
    assert_eq!(result, vec![2, 6, 10]);
}

#[tokio::test]
async fn filter_map_type_conversion() {
    let result: Vec<String> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter_map(|x| if x % 2 == 0 { Some(format!("even{}", x)) } else { None })
        .collect()
        .await;
    assert_eq!(result, vec!["even2", "even4"]);
}

#[tokio::test]
async fn filter_map_string_to_number() {
    let result: Vec<usize> = stream::iter(vec!["hello", "a", "world", "xy"])
        .filter_map(|s| if s.len() > 2 { Some(s.len()) } else { None })
        .collect()
        .await;
    assert_eq!(result, vec![5, 5]);
}

#[tokio::test]
async fn filter_map_with_mutation() {
    let mut counter = 0;
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5])
        .filter_map(|x| {
            counter += 1;
            if x % 2 == 0 { Some(x * 3) } else { None }
        })
        .collect()
        .await;
    assert_eq!(result, vec![6, 12]);
    assert_eq!(counter, 5); // All items should be processed
}

#[tokio::test]
async fn filter_map_debug_format() {
    let filter_map_stream = stream::iter(vec![1, 2, 3])
        .filter_map(|x| Some(x * 2));
    let debug_str = format!("{:?}", filter_map_stream);
    assert!(debug_str.contains("FilterMap"));
}

#[tokio::test]
async fn filter_map_size_hint() {
    let filter_map_stream = stream::iter(vec![1, 2, 3, 4, 5])
        .filter_map(|x| if x > 0 { Some(x) } else { None });
    let (lower, upper) = filter_map_stream.size_hint();
    assert_eq!(lower, 0); // Can't know lower bound due to filtering
    assert_eq!(upper, Some(5)); // Upper bound is from original stream
}

#[tokio::test]
async fn filter_map_pending_stream() {
    let mut task = task::spawn(async {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Send some items
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        tx.send(4).unwrap();
        drop(tx); // Close the stream
        
        tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
            .filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None })
            .collect::<Vec<_>>()
            .await
    });
    
    let result = assert_ready!(task.poll());
    assert_eq!(result, vec![20, 40]);
}

#[tokio::test]
async fn filter_map_option_parsing() {
    // Example: parsing strings to numbers
    let result: Vec<i32> = stream::iter(vec!["1", "not_a_number", "3", "invalid", "5"])
        .filter_map(|s| s.parse::<i32>().ok())
        .collect()
        .await;
    assert_eq!(result, vec![1, 3, 5]);
}

#[tokio::test]
async fn filter_chaining() {
    // Test chaining filter operations
    let result: Vec<i32> = stream::iter(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .filter(|&x| x % 2 == 0) // Even numbers: 2, 4, 6, 8, 10
        .filter(|&x| x > 5)      // Greater than 5: 6, 8, 10
        .collect()
        .await;
    assert_eq!(result, vec![6, 8, 10]);
}

#[tokio::test]
async fn filter_map_chaining() {
    // Test chaining filter_map operations
    let result: Vec<String> = stream::iter(vec![1, 2, 3, 4, 5, 6])
        .filter_map(|x| if x % 2 == 0 { Some(x) } else { None }) // Even: 2, 4, 6
        .filter_map(|x| if x > 3 { Some(format!("num_{}", x)) } else { None }) // >3: 4, 6
        .collect()
        .await;
    assert_eq!(result, vec!["num_4", "num_6"]);
}

#[tokio::test]
async fn filter_and_filter_map_combination() {
    // Test combining filter and filter_map
    let result: Vec<String> = stream::iter(vec![-2, -1, 0, 1, 2, 3, 4, 5])
        .filter(|&x| x > 0)                                    // Positive: 1, 2, 3, 4, 5
        .filter_map(|x| if x % 2 == 1 { Some(format!("odd_{}", x)) } else { None }) // Odd: 1, 3, 5
        .collect()
        .await;
    assert_eq!(result, vec!["odd_1", "odd_3", "odd_5"]);
}