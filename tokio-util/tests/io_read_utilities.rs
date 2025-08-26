#![warn(rust_2018_idioms)]
#![cfg(feature = "io-util")]

use bytes::BytesMut;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio_test::assert_ok;
use tokio_util::io::{read_buf, read_exact_arc};

// Mock reader that can be configured to return specific data patterns
struct MockReader {
    data: Vec<u8>,
    position: usize,
    read_size: Option<usize>,     // If set, limits read size per call
    error_on_read: Option<usize>, // If set, returns error after N reads
    reads_count: usize,
}

impl MockReader {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
            read_size: None,
            error_on_read: None,
            reads_count: 0,
        }
    }

    fn with_read_size(mut self, read_size: usize) -> Self {
        self.read_size = Some(read_size);
        self
    }

    fn with_error_after_reads(mut self, error_after: usize) -> Self {
        self.error_on_read = Some(error_after);
        self
    }
}

impl AsyncRead for MockReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.reads_count += 1;

        // Check if we should error
        if let Some(error_after) = self.error_on_read {
            if self.reads_count > error_after {
                return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "Mock error")));
            }
        }

        let remaining_data = &self.data[self.position..];
        if remaining_data.is_empty() {
            return Poll::Ready(Ok(()));
        }

        let max_read = match self.read_size {
            Some(size) => size.min(remaining_data.len()).min(buf.remaining()),
            None => remaining_data.len().min(buf.remaining()),
        };

        buf.put_slice(&remaining_data[..max_read]);
        self.position += max_read;

        Poll::Ready(Ok(()))
    }
}

// Tests for read_buf function
#[tokio::test]
async fn test_read_buf_basic() {
    let data = vec![1, 2, 3, 4, 5];
    let mut reader = MockReader::new(data.clone());
    let mut buf = BytesMut::new();

    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);

    assert_eq!(n, 5);
    assert_eq!(&buf[..], &data[..]);
}

#[tokio::test]
async fn test_read_buf_empty_reader() {
    let mut reader = MockReader::new(vec![]);
    let mut buf = BytesMut::new();

    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);

    assert_eq!(n, 0);
    assert_eq!(buf.len(), 0);
}

#[tokio::test]
async fn test_read_buf_partial_reads() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let mut reader = MockReader::new(data.clone()).with_read_size(3);
    let mut buf = BytesMut::new();

    // First read should get 3 bytes
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 3);
    assert_eq!(&buf[..], &[1, 2, 3]);

    // Second read should get next 3 bytes
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 3);
    assert_eq!(&buf[..], &[1, 2, 3, 4, 5, 6]);

    // Third read should get remaining 2 bytes
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 2);
    assert_eq!(&buf[..], &data[..]);

    // Fourth read should return 0 (EOF)
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 0);
}

#[tokio::test]
async fn test_read_buf_with_error() {
    let data = vec![1, 2, 3];
    let mut reader = MockReader::new(data).with_error_after_reads(1);
    let mut buf = BytesMut::new();

    // First read should succeed
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 3);

    // Second read should fail
    let result = read_buf(&mut reader, &mut buf).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);
}

#[tokio::test]
async fn test_read_buf_large_data() {
    let data = vec![42u8; 10000];
    let mut reader = MockReader::new(data.clone());
    let mut buf = BytesMut::new();
    let mut total_read = 0;

    // Read in chunks until all data is read
    loop {
        let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
        if n == 0 {
            break;
        }
        total_read += n;
    }

    assert_eq!(total_read, 10000);
    assert_eq!(&buf[..], &data[..]);
}

#[tokio::test]
async fn test_read_buf_incremental_growth() {
    let data = vec![1, 2, 3, 4, 5];
    let mut reader = MockReader::new(data.clone()).with_read_size(1);
    let mut buf = BytesMut::new();
    let mut total_read = 0;

    // Read byte by byte
    while total_read < data.len() {
        let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
        if n == 0 {
            break;
        }
        total_read += n;
    }

    assert_eq!(total_read, 5);
    assert_eq!(&buf[..], &data[..]);
}

#[tokio::test]
async fn test_read_buf_different_buffer_types() {
    let data = vec![10, 20, 30, 40];
    let mut reader = MockReader::new(data.clone());

    // Test with Vec<u8>
    let mut vec_buf = Vec::new();
    let n = assert_ok!(read_buf(&mut reader, &mut vec_buf).await);
    assert_eq!(n, 4);
    assert_eq!(&vec_buf[..], &data[..]);
}

// Tests for read_exact_arc function
#[tokio::test]
async fn test_read_exact_arc_basic() {
    let data = vec![1, 2, 3, 4, 5];
    let reader = MockReader::new(data.clone());

    let arc = assert_ok!(read_exact_arc(reader, 5).await);

    assert_eq!(&arc[..], &data[..]);
    // Verify it's actually an Arc by checking reference count behavior
    let arc2 = Arc::clone(&arc);
    assert_eq!(&arc2[..], &data[..]);
}

#[tokio::test]
async fn test_read_exact_arc_empty() {
    let reader = MockReader::new(vec![]);

    let arc = assert_ok!(read_exact_arc(reader, 0).await);

    assert_eq!(arc.len(), 0);
}

#[tokio::test]
async fn test_read_exact_arc_partial_reads() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let reader = MockReader::new(data.clone()).with_read_size(2);

    let arc = assert_ok!(read_exact_arc(reader, 6).await);

    assert_eq!(&arc[..], &data[..]);
}

#[tokio::test]
async fn test_read_exact_arc_insufficient_data() {
    let data = vec![1, 2, 3];
    let reader = MockReader::new(data);

    let result = read_exact_arc(reader, 5).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    assert_eq!(err.to_string(), "early eof");
}

#[tokio::test]
async fn test_read_exact_arc_exact_match() {
    let data = vec![42u8; 100];
    let reader = MockReader::new(data.clone());

    let arc = assert_ok!(read_exact_arc(reader, 100).await);

    assert_eq!(arc.len(), 100);
    assert_eq!(&arc[..], &data[..]);
}

#[tokio::test]
async fn test_read_exact_arc_large_data() {
    let data = vec![123u8; 10000];
    let reader = MockReader::new(data.clone());

    let arc = assert_ok!(read_exact_arc(reader, 10000).await);

    assert_eq!(arc.len(), 10000);
    assert_eq!(&arc[..], &data[..]);
}

#[tokio::test]
async fn test_read_exact_arc_single_byte() {
    let data = vec![255];
    let reader = MockReader::new(data.clone());

    let arc = assert_ok!(read_exact_arc(reader, 1).await);

    assert_eq!(arc.len(), 1);
    assert_eq!(arc[0], 255);
}

#[tokio::test]
async fn test_read_exact_arc_with_io_error() {
    let data = vec![1, 2, 3];
    // Force multiple reads by limiting read size, then error on second read
    let reader = MockReader::new(data)
        .with_read_size(2)
        .with_error_after_reads(1);

    let result = read_exact_arc(reader, 5).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert_eq!(err.to_string(), "Mock error");
}

#[tokio::test]
async fn test_read_exact_arc_multiple_chunks() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let reader = MockReader::new(data.clone()).with_read_size(3);

    let arc = assert_ok!(read_exact_arc(reader, 10).await);

    assert_eq!(&arc[..], &data[..]);
}

// Integration tests combining both functions
#[tokio::test]
async fn test_read_buf_then_read_exact_arc() {
    let data1 = vec![1, 2, 3];
    let data2 = vec![4, 5, 6, 7, 8];

    let mut reader1 = MockReader::new(data1.clone());
    let reader2 = MockReader::new(data2.clone());

    // First use read_buf
    let mut buf = BytesMut::new();
    let n = assert_ok!(read_buf(&mut reader1, &mut buf).await);
    assert_eq!(n, 3);
    assert_eq!(&buf[..], &data1[..]);

    // Then use read_exact_arc
    let arc = assert_ok!(read_exact_arc(reader2, 5).await);
    assert_eq!(&arc[..], &data2[..]);
}

#[tokio::test]
async fn test_read_buf_with_real_tokio_readers() {
    // Test with tokio::io::repeat
    let mut repeat_reader = tokio::io::repeat(42);
    let mut buf = BytesMut::new();

    let n = assert_ok!(read_buf(&mut repeat_reader, &mut buf).await);
    assert!(n > 0);
    assert!(buf.iter().all(|&b| b == 42));
}

#[tokio::test]
async fn test_read_exact_arc_with_real_tokio_readers() {
    // Test with tokio::io::repeat
    let repeat_reader = tokio::io::repeat(123);

    let arc = assert_ok!(read_exact_arc(repeat_reader, 50).await);
    assert_eq!(arc.len(), 50);
    assert!(arc.iter().all(|&b| b == 123));
}

// Performance and edge case tests
#[tokio::test]
async fn test_read_buf_with_zero_capacity_buffer() {
    let data = vec![1, 2, 3];
    let mut reader = MockReader::new(data);
    let mut buf = BytesMut::with_capacity(0);

    // Buffer will need to grow
    let n = assert_ok!(read_buf(&mut reader, &mut buf).await);
    assert_eq!(n, 3);
    assert_eq!(&buf[..], &[1, 2, 3]);
}

#[tokio::test]
async fn test_read_exact_arc_boundary_values() {
    // Test with various sizes including powers of 2
    let sizes = vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

    for size in sizes {
        let data = vec![size as u8; size];
        let reader = MockReader::new(data.clone());

        let arc = assert_ok!(read_exact_arc(reader, size).await);
        assert_eq!(arc.len(), size);
        assert!(arc.iter().all(|&b| b == size as u8));
    }
}
