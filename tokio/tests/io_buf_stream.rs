#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite,
    AsyncWriteExt, BufReader, BufStream, BufWriter, ReadBuf, SeekFrom,
};

use futures::task::{Context, Poll};
use std::io::{self, Cursor, IoSlice};
use std::pin::Pin;
use tokio_test::assert_ok;

// Mock stream for testing - supports both read and write
#[derive(Debug)]
struct MockStream {
    read_data: Cursor<Vec<u8>>,
    write_data: Vec<u8>,
    write_pending: bool,
    read_pending: bool,
}

impl MockStream {
    fn new(read_data: Vec<u8>) -> Self {
        Self {
            read_data: Cursor::new(read_data),
            write_data: Vec::new(),
            write_pending: false,
            read_pending: false,
        }
    }

    fn with_pending(mut self, read_pending: bool, write_pending: bool) -> Self {
        self.read_pending = read_pending;
        self.write_pending = write_pending;
        self
    }

    fn written_data(&self) -> &[u8] {
        &self.write_data
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.read_pending {
            self.read_pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        Pin::new(&mut self.read_data).poll_read(cx, buf)
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.write_pending {
            self.write_pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.write_data.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        if self.write_pending {
            self.write_pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let mut total = 0;
        for buf in bufs {
            self.write_data.extend_from_slice(buf);
            total += buf.len();
        }
        Poll::Ready(Ok(total))
    }

    fn is_write_vectored(&self) -> bool {
        true // Our mock supports vectored writes
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncSeek for MockStream {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> io::Result<()> {
        Pin::new(&mut self.read_data).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        Pin::new(&mut self.read_data).poll_complete(cx)
    }
}

#[tokio::test]
async fn new_constructor() {
    let mock = MockStream::new(b"hello world".to_vec());
    let buf_stream = BufStream::new(mock);

    // Test that we can read from it
    let mut output = String::new();
    let mut buf_stream = buf_stream;
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "hello world");
}

#[tokio::test]
async fn with_capacity_constructor() {
    let mock = MockStream::new(b"test data".to_vec());
    let buf_stream = BufStream::with_capacity(1024, 512, mock);

    // Test that we can read from it
    let mut output = String::new();
    let mut buf_stream = buf_stream;
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "test data");
}

#[tokio::test]
async fn basic_read_write() {
    let mock = MockStream::new(b"hello".to_vec());
    let mut buf_stream = BufStream::new(mock);

    // Test write
    assert_ok!(buf_stream.write_all(b"world").await);
    assert_ok!(buf_stream.flush().await);

    // Test read
    let mut output = String::new();
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "hello");

    // Check written data
    let written = buf_stream.get_ref().written_data();
    assert_eq!(written, b"world");
}

#[tokio::test]
async fn vectored_write() {
    let mock = MockStream::new(Vec::new());
    let mut buf_stream = BufStream::new(mock);

    let bufs = &[IoSlice::new(b"hello"), IoSlice::new(b" "), IoSlice::new(b"world")];
    assert_ok!(buf_stream.write_vectored(bufs).await);
    assert_ok!(buf_stream.flush().await);

    let written = buf_stream.get_ref().written_data();
    assert_eq!(written, b"hello world");
}

#[tokio::test]
async fn get_ref_access() {
    let mock = MockStream::new(b"test".to_vec());
    let buf_stream = BufStream::new(mock);

    // Test immutable reference access
    let mock_ref = buf_stream.get_ref();
    assert_eq!(mock_ref.written_data(), &[]);
}

#[tokio::test]
async fn get_mut_access() {
    let mock = MockStream::new(b"test".to_vec());
    let mut buf_stream = BufStream::new(mock);

    // Test mutable reference access
    {
        let mock_mut = buf_stream.get_mut();
        mock_mut.write_data.extend_from_slice(b"modified");
    }

    let written = buf_stream.get_ref().written_data();
    assert_eq!(written, b"modified");
}

#[tokio::test]
async fn get_pin_mut_access() {
    let mock = MockStream::new(b"test".to_vec());
    let mut buf_stream = BufStream::new(mock);

    // Test pinned mutable reference access
    let pin_mut = Pin::new(&mut buf_stream);
    let _mock_pin_mut = pin_mut.get_pin_mut();
    // Just verify it compiles and provides pinned access
}

#[tokio::test]
async fn into_inner() {
    let mock = MockStream::new(b"test".to_vec());
    let mut buf_stream = BufStream::new(mock);

    // Write some data first
    assert_ok!(buf_stream.write_all(b"written").await);
    assert_ok!(buf_stream.flush().await);

    // Convert back to inner
    let inner = buf_stream.into_inner();
    assert_eq!(inner.written_data(), b"written");
}

#[tokio::test]
async fn from_buf_reader_buf_writer() {
    let mock = MockStream::new(b"hello".to_vec());
    let buf_reader = BufReader::new(BufWriter::new(mock));
    let mut buf_stream = BufStream::from(buf_reader);

    // Test it works
    let mut output = String::new();
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "hello");
}

#[tokio::test]
async fn from_buf_writer_buf_reader() {
    let mock = MockStream::new(b"hello".to_vec());
    let buf_writer = BufWriter::new(BufReader::new(mock));
    let mut buf_stream = BufStream::from(buf_writer);

    // Test it works - this tests the more complex "invert" logic
    let mut output = String::new();
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "hello");
}

#[tokio::test]
async fn buffered_read_operations() {
    let data = b"line1\nline2\nline3\n".to_vec();
    let mock = MockStream::new(data);
    let mut buf_stream = BufStream::new(mock);

    // Test buffered read operations
    let mut line = String::new();
    assert_ok!(buf_stream.read_line(&mut line).await);
    assert_eq!(line, "line1\n");

    line.clear();
    assert_ok!(buf_stream.read_line(&mut line).await);
    assert_eq!(line, "line2\n");
}

#[tokio::test]
async fn fill_buf_and_consume() {
    let data = b"test data for buffering".to_vec();
    let mock = MockStream::new(data);
    let mut buf_stream = BufStream::new(mock);

    // Test fill_buf
    let filled = assert_ok!(buf_stream.fill_buf().await);
    assert!(!filled.is_empty());
    assert!(filled.starts_with(b"test"));

    // Test consume
    let len = filled.len().min(4);
    buf_stream.consume(len);

    // Read remaining data
    let mut remaining = String::new();
    assert_ok!(buf_stream.read_to_string(&mut remaining).await);
    assert_eq!(remaining, " data for buffering");
}

#[tokio::test]
async fn async_seek_functionality() {
    let data = b"0123456789".to_vec();
    let mock = MockStream::new(data);
    let mut buf_stream = BufStream::new(mock);

    // Seek to position 5
    let pos = assert_ok!(buf_stream.seek(SeekFrom::Start(5)).await);
    assert_eq!(pos, 5);

    // Read from position 5
    let mut output = String::new();
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "56789");
}

#[tokio::test]
async fn mixed_read_write_operations() {
    let mock = MockStream::new(b"initial data".to_vec());
    let mut buf_stream = BufStream::new(mock);

    // Read some data
    let mut buffer = [0u8; 7];
    let n = assert_ok!(buf_stream.read(&mut buffer).await);
    assert_eq!(n, 7);
    assert_eq!(&buffer[..n], b"initial");

    // Write some data
    assert_ok!(buf_stream.write_all(b" written").await);
    
    // Continue reading
    let mut remaining = String::new();
    assert_ok!(buf_stream.read_to_string(&mut remaining).await);
    assert_eq!(remaining, " data");

    // Flush to ensure write data is committed
    assert_ok!(buf_stream.flush().await);

    // Check written data
    let written = buf_stream.get_ref().written_data();
    assert_eq!(written, b" written");
}

#[tokio::test]
async fn large_data_handling() {
    // Test with data larger than typical buffer sizes
    let large_data = vec![b'x'; 8192];
    let mock = MockStream::new(large_data.clone());
    let mut buf_stream = BufStream::new(mock);

    // Write large data
    let write_data = vec![b'y'; 4096];
    assert_ok!(buf_stream.write_all(&write_data).await);

    // Read large data
    let mut read_buffer = Vec::new();
    assert_ok!(buf_stream.read_to_end(&mut read_buffer).await);
    assert_eq!(read_buffer, large_data);

    // Verify written data
    assert_ok!(buf_stream.flush().await);
    let written = buf_stream.get_ref().written_data();
    assert_eq!(written, write_data);
}

#[tokio::test]
async fn pending_operations() {
    let mock = MockStream::new(b"hello".to_vec()).with_pending(true, true);
    let mut buf_stream = BufStream::new(mock);

    // These operations should handle pending states correctly
    assert_ok!(buf_stream.write_all(b"world").await);
    assert_ok!(buf_stream.flush().await);

    let mut output = String::new();
    assert_ok!(buf_stream.read_to_string(&mut output).await);
    assert_eq!(output, "hello");
}

#[tokio::test]
async fn shutdown_operation() {
    let mock = MockStream::new(Vec::new());
    let mut buf_stream = BufStream::new(mock);

    assert_ok!(buf_stream.write_all(b"test").await);
    assert_ok!(buf_stream.shutdown().await);
}

#[tokio::test]
async fn is_write_vectored() {
    let mock = MockStream::new(Vec::new());
    let buf_stream = BufStream::new(mock);

    // Test is_write_vectored (should return true for our mock now)
    assert!(buf_stream.is_write_vectored());
}

#[tokio::test]
async fn debug_formatting() {
    let mock = MockStream::new(b"test".to_vec());
    let buf_stream = BufStream::new(mock);

    let debug_str = format!("{:?}", buf_stream);
    assert!(debug_str.contains("BufStream"));
}

#[tokio::test]
async fn error_handling() {
    // Test with a stream that can produce errors
    struct ErrorStream;

    impl AsyncRead for ErrorStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "read error")))
        }
    }

    impl AsyncWrite for ErrorStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            _: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "write error")))
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "flush error")))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let mut buf_stream = BufStream::new(ErrorStream);

    // Test read error propagation
    let mut buffer = [0u8; 10];
    let result = buf_stream.read(&mut buffer).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);

    // Test write error propagation - need to flush to see the error
    // since BufWriter buffers writes
    let _result = buf_stream.write(b"test").await;
    // This might succeed due to buffering, so let's test flush instead
    let flush_result = buf_stream.flush().await;
    assert!(flush_result.is_err());
    assert_eq!(flush_result.unwrap_err().kind(), io::ErrorKind::Other);
}