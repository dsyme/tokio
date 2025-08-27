#![warn(rust_2018_idioms)]
#![cfg(all(feature = "full", feature = "io-std"))]

use tokio::io::{self, AsyncWriteExt};
use tokio_test::assert_ok;

#[cfg(unix)]
use std::os::unix::io::{AsFd, AsRawFd};
#[cfg(windows)]
use tokio::os::windows::io::{AsHandle, AsRawHandle};

/// Test basic stdin functionality and constructor
#[tokio::test]
async fn stdin_basic_functionality() {
    // Test that stdin() constructor works
    let stdin = io::stdin();

    // Test Debug implementation
    let debug_str = format!("{:?}", stdin);
    assert!(debug_str.contains("Stdin"));
}

/// Test that multiple stdin handles can be created
#[tokio::test]
async fn stdin_multiple_handles() {
    let stdin1 = io::stdin();
    let stdin2 = io::stdin();

    // Both should be valid handles
    let debug1 = format!("{:?}", stdin1);
    let debug2 = format!("{:?}", stdin2);

    assert!(debug1.contains("Stdin"));
    assert!(debug2.contains("Stdin"));
}

/// Test stdin raw handle access on Unix platforms
#[tokio::test]
#[cfg(unix)]
async fn stdin_unix_raw_fd() {
    let stdin = io::stdin();

    // Test AsRawFd implementation
    let raw_fd = stdin.as_raw_fd();
    assert!(raw_fd >= 0); // stdin should have a valid file descriptor

    // Test AsFd implementation
    let borrowed_fd = stdin.as_fd();
    assert_eq!(borrowed_fd.as_raw_fd(), raw_fd);
}

/// Test stdin raw handle access on Windows platforms  
#[tokio::test]
#[cfg(windows)]
async fn stdin_windows_raw_handle() {
    let stdin = io::stdin();

    // Test AsRawHandle implementation
    let _raw_handle = stdin.as_raw_handle();

    // Test AsHandle implementation
    let _borrowed_handle = stdin.as_handle();
}

/// Test basic stdout functionality and constructor
#[tokio::test]
async fn stdout_basic_functionality() {
    // Test that stdout() constructor works
    let stdout = io::stdout();

    // Test Debug implementation
    let debug_str = format!("{:?}", stdout);
    assert!(debug_str.contains("Stdout"));
}

/// Test that multiple stdout handles can be created
#[tokio::test]
async fn stdout_multiple_handles() {
    let stdout1 = io::stdout();
    let stdout2 = io::stdout();

    // Both should be valid handles
    let debug1 = format!("{:?}", stdout1);
    let debug2 = format!("{:?}", stdout2);

    assert!(debug1.contains("Stdout"));
    assert!(debug2.contains("Stdout"));
}

/// Test basic stdout write operations
#[tokio::test]
async fn stdout_write_operations() {
    let mut stdout = io::stdout();

    // Test basic write - this should succeed even if output goes nowhere in tests
    assert_ok!(stdout.write(b"test message").await);

    // Test write_all
    assert_ok!(stdout.write_all(b"another test message").await);

    // Test flush
    assert_ok!(stdout.flush().await);

    // Test shutdown
    assert_ok!(stdout.shutdown().await);
}

/// Test stdout with larger data writes
#[tokio::test]
async fn stdout_large_write() {
    let mut stdout = io::stdout();

    // Write a larger chunk of data
    let large_data = vec![b'x'; 1000];
    assert_ok!(stdout.write_all(&large_data).await);
    assert_ok!(stdout.flush().await);
}

/// Test stdout vectored write operations
#[tokio::test]
async fn stdout_vectored_write() {
    let mut stdout = io::stdout();

    let buf1 = b"first part";
    let buf2 = b"second part";
    let bufs = &[std::io::IoSlice::new(buf1), std::io::IoSlice::new(buf2)][..];

    // Test vectored write
    let bytes_written = assert_ok!(stdout.write_vectored(bufs).await);
    assert!(bytes_written > 0);

    assert_ok!(stdout.flush().await);
}

/// Test stdout raw handle access on Unix platforms
#[tokio::test]
#[cfg(unix)]
async fn stdout_unix_raw_fd() {
    let stdout = io::stdout();

    // Test AsRawFd implementation
    let raw_fd = stdout.as_raw_fd();
    assert!(raw_fd >= 0); // stdout should have a valid file descriptor

    // Test AsFd implementation
    let borrowed_fd = stdout.as_fd();
    assert_eq!(borrowed_fd.as_raw_fd(), raw_fd);
}

/// Test stdout raw handle access on Windows platforms
#[tokio::test]
#[cfg(windows)]
async fn stdout_windows_raw_handle() {
    let stdout = io::stdout();

    // Test AsRawHandle implementation
    let _raw_handle = stdout.as_raw_handle();

    // Test AsHandle implementation
    let _borrowed_handle = stdout.as_handle();
}

/// Test cooperative behavior of stdout operations
#[tokio::test]
async fn stdout_cooperative_behavior() {
    // Test that stdout operations yield properly and don't starve other tasks
    tokio::select! {
        biased;
        _ = async {
            let mut stdout = io::stdout();
            for _ in 0..10 {
                let _ = stdout.write(b"test").await;
                let _ = stdout.flush().await;
            }
        } => {},
        _ = tokio::task::yield_now() => {}
    }
}

/// Test basic stderr functionality and constructor
#[tokio::test]
async fn stderr_basic_functionality() {
    // Test that stderr() constructor works
    let stderr = io::stderr();

    // Test Debug implementation
    let debug_str = format!("{:?}", stderr);
    assert!(debug_str.contains("Stderr"));
}

/// Test that multiple stderr handles can be created
#[tokio::test]
async fn stderr_multiple_handles() {
    let stderr1 = io::stderr();
    let stderr2 = io::stderr();

    // Both should be valid handles
    let debug1 = format!("{:?}", stderr1);
    let debug2 = format!("{:?}", stderr2);

    assert!(debug1.contains("Stderr"));
    assert!(debug2.contains("Stderr"));
}

/// Test basic stderr write operations
#[tokio::test]
async fn stderr_write_operations() {
    let mut stderr = io::stderr();

    // Test basic write - this should succeed even if output goes nowhere in tests
    assert_ok!(stderr.write(b"error message").await);

    // Test write_all
    assert_ok!(stderr.write_all(b"another error message").await);

    // Test flush
    assert_ok!(stderr.flush().await);

    // Test shutdown
    assert_ok!(stderr.shutdown().await);
}

/// Test stderr with larger data writes
#[tokio::test]
async fn stderr_large_write() {
    let mut stderr = io::stderr();

    // Write a larger chunk of data
    let large_data = vec![b'!'; 1000];
    assert_ok!(stderr.write_all(&large_data).await);
    assert_ok!(stderr.flush().await);
}

/// Test stderr vectored write operations
#[tokio::test]
async fn stderr_vectored_write() {
    let mut stderr = io::stderr();

    let buf1 = b"error: ";
    let buf2 = b"something went wrong";
    let bufs = &[std::io::IoSlice::new(buf1), std::io::IoSlice::new(buf2)][..];

    // Test vectored write
    let bytes_written = assert_ok!(stderr.write_vectored(bufs).await);
    assert!(bytes_written > 0);

    assert_ok!(stderr.flush().await);
}

/// Test stderr raw handle access on Unix platforms
#[tokio::test]
#[cfg(unix)]
async fn stderr_unix_raw_fd() {
    let stderr = io::stderr();

    // Test AsRawFd implementation
    let raw_fd = stderr.as_raw_fd();
    assert!(raw_fd >= 0); // stderr should have a valid file descriptor

    // Test AsFd implementation
    let borrowed_fd = stderr.as_fd();
    assert_eq!(borrowed_fd.as_raw_fd(), raw_fd);
}

/// Test stderr raw handle access on Windows platforms
#[tokio::test]
#[cfg(windows)]
async fn stderr_windows_raw_handle() {
    let stderr = io::stderr();

    // Test AsRawHandle implementation
    let _raw_handle = stderr.as_raw_handle();

    // Test AsHandle implementation
    let _borrowed_handle = stderr.as_handle();
}

/// Test cooperative behavior of stderr operations
#[tokio::test]
async fn stderr_cooperative_behavior() {
    // Test that stderr operations yield properly and don't starve other tasks
    tokio::select! {
        biased;
        _ = async {
            let mut stderr = io::stderr();
            for _ in 0..10 {
                let _ = stderr.write(b"error").await;
                let _ = stderr.flush().await;
            }
        } => {},
        _ = tokio::task::yield_now() => {}
    }
}

/// Test concurrent writes to stdout/stderr don't interfere
#[tokio::test]
async fn concurrent_stdout_stderr_writes() {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let task1 = tokio::spawn(async {
        let mut stdout = io::stdout();
        for i in 0..5 {
            let msg = format!("stdout message {}\n", i);
            let _ = stdout.write_all(msg.as_bytes()).await;
            let _ = stdout.flush().await;
        }
    });

    let task2 = tokio::spawn(async {
        let mut stderr = io::stderr();
        for i in 0..5 {
            let msg = format!("stderr message {}\n", i);
            let _ = stderr.write_all(msg.as_bytes()).await;
            let _ = stderr.flush().await;
        }
        let _ = tx.send(());
    });

    // Wait for both tasks to complete
    assert_ok!(task1.await);
    assert_ok!(task2.await);
    assert_ok!(rx.await);
}

/// Test empty write operations
#[tokio::test]
async fn empty_writes() {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    // Test empty writes should succeed
    assert_ok!(stdout.write(&[]).await);
    assert_ok!(stderr.write(&[]).await);

    assert_ok!(stdout.write_all(&[]).await);
    assert_ok!(stderr.write_all(&[]).await);

    assert_ok!(stdout.flush().await);
    assert_ok!(stderr.flush().await);
}

/// Test mixed operation patterns
#[tokio::test]
async fn mixed_operations() {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    // Mix different write patterns
    assert_ok!(stdout.write(b"hello").await);
    assert_ok!(stderr.write_all(b"error occurred").await);
    assert_ok!(stdout.flush().await);
    assert_ok!(stderr.write(b" - details").await);
    assert_ok!(stdout.write_all(b" world").await);
    assert_ok!(stderr.flush().await);
    assert_ok!(stdout.shutdown().await);
    assert_ok!(stderr.shutdown().await);
}
