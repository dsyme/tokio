#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio_test::assert_ok;

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Mock reader that returns specific byte sequences
struct MockReader {
    data: Vec<u8>,
    pos: usize,
    read_size: Option<usize>, // If Some, limits bytes returned per read
}

impl MockReader {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            pos: 0,
            read_size: None,
        }
    }

    fn with_read_size(mut self, size: usize) -> Self {
        self.read_size = Some(size);
        self
    }
}

impl AsyncRead for MockReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.pos >= self.data.len() {
            return Poll::Ready(Ok(()));
        }

        let remaining = self.data.len() - self.pos;
        let to_read = self
            .read_size
            .map(|size| size.min(remaining).min(buf.remaining()))
            .unwrap_or(remaining.min(buf.remaining()));

        if to_read == 0 {
            return Poll::Ready(Ok(()));
        }

        buf.put_slice(&self.data[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Poll::Ready(Ok(()))
    }
}

/// Mock reader that always returns an error
struct ErrorReader;

impl AsyncRead for ErrorReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, "read error")))
    }
}

/// Mock reader that returns EOF immediately
struct EmptyReader;

impl AsyncRead for EmptyReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn read_u8_success() {
    let mut reader = MockReader::new(vec![42, 255, 0, 128]);

    assert_eq!(assert_ok!(reader.read_u8().await), 42);
    assert_eq!(assert_ok!(reader.read_u8().await), 255);
    assert_eq!(assert_ok!(reader.read_u8().await), 0);
    assert_eq!(assert_ok!(reader.read_u8().await), 128);
}

#[tokio::test]
async fn read_i8_success() {
    let mut reader = MockReader::new(vec![42, 255, 0, 128]);

    assert_eq!(assert_ok!(reader.read_i8().await), 42);
    assert_eq!(assert_ok!(reader.read_i8().await), -1); // 255 as i8
    assert_eq!(assert_ok!(reader.read_i8().await), 0);
    assert_eq!(assert_ok!(reader.read_i8().await), -128); // 128 as i8
}

#[tokio::test]
async fn read_u16_big_endian() {
    let mut reader = MockReader::new(vec![0x12, 0x34, 0xFF, 0xFF]);

    assert_eq!(assert_ok!(reader.read_u16().await), 0x1234);
    assert_eq!(assert_ok!(reader.read_u16().await), 0xFFFF);
}

#[tokio::test]
async fn read_u16_little_endian() {
    let mut reader = MockReader::new(vec![0x34, 0x12, 0xFF, 0xFF]);

    assert_eq!(assert_ok!(reader.read_u16_le().await), 0x1234);
    assert_eq!(assert_ok!(reader.read_u16_le().await), 0xFFFF);
}

#[tokio::test]
async fn read_i16_big_endian() {
    let mut reader = MockReader::new(vec![0x80, 0x00, 0x7F, 0xFF]);

    assert_eq!(assert_ok!(reader.read_i16().await), -32768);
    assert_eq!(assert_ok!(reader.read_i16().await), 32767);
}

#[tokio::test]
async fn read_i16_little_endian() {
    let mut reader = MockReader::new(vec![0x00, 0x80, 0xFF, 0x7F]);

    assert_eq!(assert_ok!(reader.read_i16_le().await), -32768);
    assert_eq!(assert_ok!(reader.read_i16_le().await), 32767);
}

#[tokio::test]
async fn read_u32_big_endian() {
    let mut reader = MockReader::new(vec![0x12, 0x34, 0x56, 0x78, 0xFF, 0xFF, 0xFF, 0xFF]);

    assert_eq!(assert_ok!(reader.read_u32().await), 0x12345678);
    assert_eq!(assert_ok!(reader.read_u32().await), 0xFFFFFFFF);
}

#[tokio::test]
async fn read_u32_little_endian() {
    let mut reader = MockReader::new(vec![0x78, 0x56, 0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF]);

    assert_eq!(assert_ok!(reader.read_u32_le().await), 0x12345678);
    assert_eq!(assert_ok!(reader.read_u32_le().await), 0xFFFFFFFF);
}

#[tokio::test]
async fn read_i32_big_endian() {
    let mut reader = MockReader::new(vec![0x80, 0x00, 0x00, 0x00, 0x7F, 0xFF, 0xFF, 0xFF]);

    assert_eq!(assert_ok!(reader.read_i32().await), -2147483648);
    assert_eq!(assert_ok!(reader.read_i32().await), 2147483647);
}

#[tokio::test]
async fn read_i32_little_endian() {
    let mut reader = MockReader::new(vec![0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF, 0xFF, 0x7F]);

    assert_eq!(assert_ok!(reader.read_i32_le().await), -2147483648);
    assert_eq!(assert_ok!(reader.read_i32_le().await), 2147483647);
}

#[tokio::test]
async fn read_u64_big_endian() {
    let mut reader = MockReader::new(vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ]);

    assert_eq!(assert_ok!(reader.read_u64().await), 0x123456789ABCDEF0);
    assert_eq!(assert_ok!(reader.read_u64().await), 0xFFFFFFFFFFFFFFFF);
}

#[tokio::test]
async fn read_u64_little_endian() {
    let mut reader = MockReader::new(vec![
        0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ]);

    assert_eq!(assert_ok!(reader.read_u64_le().await), 0x123456789ABCDEF0);
    assert_eq!(assert_ok!(reader.read_u64_le().await), 0xFFFFFFFFFFFFFFFF);
}

#[tokio::test]
async fn read_i64_big_endian() {
    let mut reader = MockReader::new(vec![
        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ]);

    assert_eq!(assert_ok!(reader.read_i64().await), -9223372036854775808);
    assert_eq!(assert_ok!(reader.read_i64().await), 9223372036854775807);
}

#[tokio::test]
async fn read_i64_little_endian() {
    let mut reader = MockReader::new(vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x7F,
    ]);

    assert_eq!(assert_ok!(reader.read_i64_le().await), -9223372036854775808);
    assert_eq!(assert_ok!(reader.read_i64_le().await), 9223372036854775807);
}

#[tokio::test]
async fn read_u128_big_endian() {
    let mut reader = MockReader::new(vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88,
    ]);

    assert_eq!(
        assert_ok!(reader.read_u128().await),
        0x123456789ABCDEF01122334455667788
    );
}

#[tokio::test]
async fn read_u128_little_endian() {
    let mut reader = MockReader::new(vec![
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34,
        0x12,
    ]);

    assert_eq!(
        assert_ok!(reader.read_u128_le().await),
        0x123456789ABCDEF01122334455667788
    );
}

#[tokio::test]
async fn read_i128_big_endian() {
    let mut reader = MockReader::new(vec![
        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ]);

    assert_eq!(
        assert_ok!(reader.read_i128().await),
        -170141183460469231731687303715884105728
    );
    assert_eq!(
        assert_ok!(reader.read_i128().await),
        170141183460469231731687303715884105727
    );
}

#[tokio::test]
async fn read_i128_little_endian() {
    let mut reader = MockReader::new(vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0x7F,
    ]);

    assert_eq!(
        assert_ok!(reader.read_i128_le().await),
        -170141183460469231731687303715884105728
    );
    assert_eq!(
        assert_ok!(reader.read_i128_le().await),
        170141183460469231731687303715884105727
    );
}

#[tokio::test]
async fn read_f32_big_endian() {
    // 3.14159274 as f32 in big endian bytes
    let mut reader = MockReader::new(vec![0x40, 0x49, 0x0F, 0xDC]);

    let value = assert_ok!(reader.read_f32().await);
    assert!((value - 3.141_592_7).abs() < 0.0001);
}

#[tokio::test]
async fn read_f32_little_endian() {
    // 3.14159274 as f32 in little endian bytes
    let mut reader = MockReader::new(vec![0xDC, 0x0F, 0x49, 0x40]);

    let value = assert_ok!(reader.read_f32_le().await);
    assert!((value - 3.141_592_7).abs() < 0.0001);
}

#[tokio::test]
async fn read_f64_big_endian() {
    // PI as f64 in big endian bytes
    let mut reader = MockReader::new(vec![0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]);

    let value = assert_ok!(reader.read_f64().await);
    assert!((value - std::f64::consts::PI).abs() < f64::EPSILON);
}

#[tokio::test]
async fn read_f64_little_endian() {
    // PI as f64 in little endian bytes
    let mut reader = MockReader::new(vec![0x18, 0x2D, 0x44, 0x54, 0xFB, 0x21, 0x09, 0x40]);

    let value = assert_ok!(reader.read_f64_le().await);
    assert!((value - std::f64::consts::PI).abs() < f64::EPSILON);
}

// Error handling tests
#[tokio::test]
async fn read_u8_unexpected_eof() {
    let mut reader = EmptyReader;

    let result = reader.read_u8().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_i8_unexpected_eof() {
    let mut reader = EmptyReader;

    let result = reader.read_i8().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_u16_unexpected_eof() {
    let mut reader = MockReader::new(vec![0x12]); // Only 1 byte, need 2

    let result = reader.read_u16().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_u32_unexpected_eof() {
    let mut reader = MockReader::new(vec![0x12, 0x34, 0x56]); // Only 3 bytes, need 4

    let result = reader.read_u32().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_u64_unexpected_eof() {
    let mut reader = MockReader::new(vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]); // Only 6 bytes, need 8

    let result = reader.read_u64().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_u128_unexpected_eof() {
    let mut reader = MockReader::new(vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
    ]); // Only 14 bytes, need 16

    let result = reader.read_u128().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

#[tokio::test]
async fn read_u8_io_error() {
    let mut reader = ErrorReader;

    let result = reader.read_u8().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);
}

#[tokio::test]
async fn read_u16_io_error() {
    let mut reader = ErrorReader;

    let result = reader.read_u16().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Other);
}

// Incremental read tests (testing partial reads)
#[tokio::test]
async fn read_u16_incremental() {
    let mut reader = MockReader::new(vec![0x12, 0x34]).with_read_size(1);

    assert_eq!(assert_ok!(reader.read_u16().await), 0x1234);
}

#[tokio::test]
async fn read_u32_incremental() {
    let mut reader = MockReader::new(vec![0x12, 0x34, 0x56, 0x78]).with_read_size(1);

    assert_eq!(assert_ok!(reader.read_u32().await), 0x12345678);
}

#[tokio::test]
async fn read_u64_incremental() {
    let mut reader =
        MockReader::new(vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]).with_read_size(3);

    assert_eq!(assert_ok!(reader.read_u64().await), 0x123456789ABCDEF0);
}

#[tokio::test]
async fn read_u128_incremental() {
    let mut reader = MockReader::new(vec![
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88,
    ])
    .with_read_size(5);

    assert_eq!(
        assert_ok!(reader.read_u128().await),
        0x123456789ABCDEF01122334455667788
    );
}

// Mixed types reading test
#[tokio::test]
async fn read_mixed_types() {
    let mut reader = MockReader::new(vec![
        42,  // u8
        255, // i8 (-1)
        0x12, 0x34, // u16 big endian (0x1234)
        0x78, 0x56, 0x34, 0x12, // u32 little endian (0x12345678)
        0x40, 0x49, 0x0F, 0xDC, // f32 big endian (3.14159274)
    ]);

    assert_eq!(assert_ok!(reader.read_u8().await), 42);
    assert_eq!(assert_ok!(reader.read_i8().await), -1);
    assert_eq!(assert_ok!(reader.read_u16().await), 0x1234);
    assert_eq!(assert_ok!(reader.read_u32_le().await), 0x12345678);

    let f_value = assert_ok!(reader.read_f32().await);
    assert!((f_value - 3.141_592_7).abs() < 0.0001);
}

// Edge case: Zero values
#[tokio::test]
async fn read_zero_values() {
    let mut reader = MockReader::new(vec![0; 32]); // 32 zero bytes

    assert_eq!(assert_ok!(reader.read_u8().await), 0);
    assert_eq!(assert_ok!(reader.read_i8().await), 0);
    assert_eq!(assert_ok!(reader.read_u16().await), 0);
    assert_eq!(assert_ok!(reader.read_i16().await), 0);
    assert_eq!(assert_ok!(reader.read_u32().await), 0);
    assert_eq!(assert_ok!(reader.read_i32().await), 0);
    assert_eq!(assert_ok!(reader.read_u64().await), 0);
    assert_eq!(assert_ok!(reader.read_i64().await), 0);
}

// Edge case: Maximum values
#[tokio::test]
async fn read_max_values() {
    let mut reader = MockReader::new(vec![0xFF; 16]);

    assert_eq!(assert_ok!(reader.read_u8().await), u8::MAX);
    assert_eq!(assert_ok!(reader.read_i8().await), -1); // 0xFF as i8
    assert_eq!(assert_ok!(reader.read_u16().await), u16::MAX);
    assert_eq!(assert_ok!(reader.read_u32().await), u32::MAX);
    assert_eq!(assert_ok!(reader.read_u64().await), u64::MAX);
}
