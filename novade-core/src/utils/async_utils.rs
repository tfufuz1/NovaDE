//! Asynchronous Utilities for NovaDE Core.
//!
//! This module provides helper functions and wrappers for common asynchronous operations,
//! primarily leveraging the `tokio` runtime. It aims to offer consistent and convenient
//! interfaces for tasks like spawning futures, handling timeouts, sleeping, and creating intervals.
//!
//! # Dependencies
//!
//! This module relies on the `tokio` crate for its asynchronous runtime capabilities.
//! Ensure `tokio` is a dependency in your project if you use these utilities directly
//! or if `novade-core` is built with features that enable them.
//!
//! # Key Functions
//!
//! - [`spawn_task`]: A wrapper around `tokio::spawn` for launching new asynchronous tasks.
//! - [`timeout`]: Executes a future with a specified timeout, returning an error if the
//!   timeout elapses before the future completes.
//! - [`sleep`]: Asynchronously pauses execution for a given duration.
//! - [`interval`]: Creates a stream that yields at a regular periodic interval.
//!
//! # Examples
//!
//! ```rust,ignore
//! use novade_core::utils::async_utils::{spawn_task, timeout, sleep};
//! use std::time::Duration;
//!
//! async fn my_async_function() -> u32 {
//!     sleep(Duration::from_millis(50)).await;
//!     42
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Spawn a task
//!     let handle = spawn_task(async {
//!         println!("Task running in background!");
//!         my_async_function().await
//!     });
//!
//!     // Run a future with a timeout
//!     match timeout(Duration::from_millis(100), my_async_function()).await {
//!         Ok(Ok(value)) => println!("my_async_function completed with: {}", value),
//!         Ok(Err(_timeout_error)) => eprintln!("my_async_function timed out!"),
//!         Err(_) => eprintln!("An internal error occurred with the timeout itself."), // Should not happen with tokio::time::timeout
//!     }
//!
//!     let result = handle.await.expect("Task panicked");
//!     println!("Background task result: {}", result);
//! }
//! ```
//! Note: The example above for `timeout` shows `Result<Result<T, time::error::Elapsed>, JoinError>`
//! if you were to `spawn_task` the `timeout` itself. Direct await as in the tests is `Result<T, time::error::Elapsed>`.
//! The example provided in the file `async fn main` block is more direct.

use std::future::Future;
use std::time::Duration;
use tokio::task::{JoinHandle, spawn};
use tokio::time;

/// Spawns a new asynchronous task on the Tokio runtime.
///
/// This function is a direct wrapper around `tokio::spawn`. It takes a future
/// and executes it on the runtime, returning a `JoinHandle` that can be used
/// to await the task's completion.
///
/// The future must be `Send` and its output must be `Send` to allow it to be
/// moved between threads if the runtime's scheduler deems it necessary.
///
/// # Type Parameters
///
/// * `F`: The type of the future to spawn.
///
/// # Arguments
///
/// * `future`: The future to be spawned. It must be `'static` to ensure it lives
///   long enough for the task to complete, even if the original handle is dropped.
///
/// # Returns
///
/// A [`JoinHandle<F::Output>`] which can be `.await`ed to get the result of the future.
///
/// # Examples
///
/// ```
/// use novade_core::utils::async_utils::spawn_task;
///
/// #[tokio::main]
/// async fn main() {
///     let handle = spawn_task(async {
///         // Some background work
///         "done".to_string()
///     });
///     let result = handle.await.unwrap();
///     assert_eq!(result, "done");
/// }
/// ```
pub fn spawn_task<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    spawn(future)
}

/// Executes a future with a specified timeout.
///
/// This function wraps `tokio::time::timeout`. If the provided `future` completes
/// before the `duration` elapses, its result is returned as `Ok(T)`. If the
/// timeout occurs first, an `Err(tokio::time::error::Elapsed)` is returned.
///
/// # Type Parameters
///
/// * `F`: The type of the future to run with a timeout.
/// * `T`: The output type of the future `F`.
///
/// # Arguments
///
/// * `duration`: The maximum `std::time::Duration` to wait for the future to complete.
/// * `future`: The future to execute.
///
/// # Returns
///
/// A `Result` which is:
/// - `Ok(T)`: If the future `future` completes successfully within `duration`.
/// - `Err(tokio::time::error::Elapsed)`: If the `duration` elapses before `future` completes.
///
/// # Examples
///
/// ```
/// use novade_core::utils::async_utils::{timeout, sleep};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     // A future that completes in time
///     let fast_future = async { "finished" };
///     match timeout(Duration::from_secs(1), fast_future).await {
///         Ok(result) => assert_eq!(result, "finished"),
///         Err(_) => panic!("Fast future should not time out"),
///     }
///
///     // A future that times out
///     let slow_future = async {
///         sleep(Duration::from_secs(2)).await;
///         "finished slowly"
///     };
///     match timeout(Duration::from_millis(100), slow_future).await {
///         Ok(_) => panic!("Slow future should have timed out"),
///         Err(e) => {
///             // Correctly identify the timeout error
///             assert!(matches!(e, tokio::time::error::Elapsed { .. }));
///         }
///     }
/// }
/// ```
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, time::error::Elapsed>
where
    F: Future<Output = T>,
{
    time::timeout(duration, future).await
}

/// Pauses the current asynchronous task for the specified duration.
///
/// This function is a direct wrapper around `tokio::time::sleep`.
/// The task will resume execution after the `duration` has elapsed.
///
/// # Arguments
///
/// * `duration`: The `std::time::Duration` for which to sleep.
///
/// # Examples
///
/// ```
/// use novade_core::utils::async_utils::sleep;
/// use std::time::{Duration, Instant};
///
/// #[tokio::main]
/// async fn main() {
///     let start = Instant::now();
///     sleep(Duration::from_millis(100)).await;
///     let elapsed = start.elapsed();
///     assert!(elapsed >= Duration::from_millis(100));
/// }
/// ```
pub async fn sleep(duration: Duration) {
    time::sleep(duration).await
}

/// Creates a stream that yields at a regular periodic interval.
///
/// This function is a direct wrapper around `tokio::time::interval`.
/// The first tick completes immediately. Subsequent ticks will complete after
/// the specified `period` has elapsed from the previous tick.
///
/// # Arguments
///
/// * `period`: The `std::time::Duration` between ticks.
///
/// # Returns
///
/// A [`tokio::time::Interval`] which can be `.tick().await`ed in a loop.
///
/// # Examples
///
/// ```
/// use novade_core::utils::async_utils::interval;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let mut interval = interval(Duration::from_millis(50));
///     interval.tick().await; // First tick is immediate
///
///     let mut tick_count = 0;
///     for _ in 0..3 {
///         interval.tick().await;
///         tick_count += 1;
///     }
///     assert_eq!(tick_count, 3);
/// }
/// ```
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
