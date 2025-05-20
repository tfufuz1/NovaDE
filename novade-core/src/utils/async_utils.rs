//! Async utilities for the NovaDE core layer.
//!
//! This module provides asynchronous utilities used throughout the
//! NovaDE desktop environment.

use std::future::Future;
use std::time::Duration;
use tokio::task::{JoinHandle, spawn};
use tokio::time;

/// Spawns a task on the Tokio runtime.
///
/// This function is a wrapper around `tokio::spawn` that provides
/// a consistent interface for spawning tasks.
///
/// # Arguments
///
/// * `future` - The future to spawn
///
/// # Returns
///
/// A `JoinHandle` for the spawned task.
pub fn spawn_task<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    spawn(future)
}

/// Runs a future with a timeout.
///
/// This function is a wrapper around `tokio::time::timeout` that provides
/// a consistent interface for running futures with timeouts.
///
/// # Arguments
///
/// * `duration` - The timeout duration
/// * `future` - The future to run
///
/// # Returns
///
/// A future that resolves to `Ok(T)` if the future completes within the timeout,
/// or `Err(tokio::time::error::Elapsed)` if the timeout expires.
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, time::error::Elapsed>
where
    F: Future<Output = T>,
{
    time::timeout(duration, future).await
}

/// Sleeps for the specified duration.
///
/// This function is a wrapper around `tokio::time::sleep` that provides
/// a consistent interface for sleeping.
///
/// # Arguments
///
/// * `duration` - The duration to sleep
pub async fn sleep(duration: Duration) {
    time::sleep(duration).await
}

/// Creates a repeating interval.
///
/// This function is a wrapper around `tokio::time::interval` that provides
/// a consistent interface for creating intervals.
///
/// # Arguments
///
/// * `period` - The interval period
///
/// # Returns
///
/// A `tokio::time::Interval` that yields at the specified interval.
pub fn interval(period: Duration) -> time::Interval {
    time::interval(period)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spawn_task() {
        let handle = spawn_task(async {
            42
        });
        
        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }
    
    #[tokio::test]
    async fn test_timeout_success() {
        let result = timeout(Duration::from_secs(1), async {
            42
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[tokio::test]
    async fn test_timeout_elapsed() {
        let result = timeout(Duration::from_millis(10), async {
            sleep(Duration::from_millis(50)).await;
            42
        }).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_sleep() {
        let start = std::time::Instant::now();
        sleep(Duration::from_millis(50)).await;
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(50));
    }
    
    #[tokio::test]
    async fn test_interval() {
        let mut interval = interval(Duration::from_millis(10));
        
        // First tick completes immediately
        interval.tick().await;
        
        let start = std::time::Instant::now();
        interval.tick().await;
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(10));
    }
}
