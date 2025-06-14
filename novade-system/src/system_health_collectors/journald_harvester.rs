//! # Journald Log Harvester
//!
//! This module provides an implementation of the `LogHarvester` trait for
//! reading log entries from the systemd journal (`journald`).
//! It uses the `sd-journal` crate to interact with the journal.

use async_trait::async_trait;
use futures_core::Stream;
use novade_core::types::system_health::{LogEntry, LogFilter, LogLevel, TimeRange};
use crate::error::SystemError;
use crate::system_health_collectors::LogHarvester;
use sd_journal::{Journal, JournalEntry, JournalSeek};
use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use async_stream::stream; // For the polling-based stream implementation.

/// A log harvester that reads log entries from the systemd journal.
///
/// Implements the `LogHarvester` trait. It can query historical logs with filtering
/// and provides a basic polling-based stream for live log entries.
/// Assumes that the system is running systemd and journald is accessible.
pub struct JournaldLogHarvester;

/// Maps a journal priority value (0-7, syslog convention) to a `LogLevel`.
fn map_priority_to_loglevel(priority: u8) -> LogLevel {
    match priority {
        0 => LogLevel::Emergency, // System is unusable
        1 => LogLevel::Alert,     // Action must be taken immediately
        2 => LogLevel::Critical,  // Critical conditions
        3 => LogLevel::Error,     // Error conditions
        4 => LogLevel::Warning,   // Warning conditions
        5 => LogLevel::Notice,    // Normal but significant condition
        6 => LogLevel::Info,      // Informational messages
        7 => LogLevel::Debug,     // Debug-level messages
        _ => LogLevel::Unknown,   // Default for any other value (should not happen with valid journal entries)
    }
}

/// Converts a `sd_journal::JournalEntry` to a `novade_core::types::system_health::LogEntry`.
///
/// Extracts fields like `MESSAGE`, `PRIORITY`, `__REALTIME_TIMESTAMP`, `SYSLOG_IDENTIFIER`,
/// `_COMM`, `_SYSTEMD_UNIT`, and `_HOSTNAME` from the journal entry and maps them
/// to the corresponding fields in `LogEntry`.
fn journal_entry_to_log_entry(entry: &JournalEntry) -> Result<LogEntry, SystemError> {
    let message = entry.get("MESSAGE")
        .unwrap_or_else(|| "No message".to_string()); // Default if MESSAGE field is missing
    let priority = entry.get("PRIORITY")
        .and_then(|p_str| p_str.parse::<u8>().ok())
        .unwrap_or(7); // Default to Debug if PRIORITY is missing or not parsable

    // Timestamps in journald are typically microseconds since UNIX_EPOCH.
    let timestamp_us_str = entry.get("__REALTIME_TIMESTAMP")
        .ok_or_else(|| SystemError::MetricCollectorError("Missing __REALTIME_TIMESTAMP".to_string()))?;
    let timestamp_us = timestamp_us_str.parse::<u64>()
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse timestamp: {}", e)))?;

    let system_time = UNIX_EPOCH + Duration::from_micros(timestamp_us);

    let component = entry.get("SYSLOG_IDENTIFIER")
        .or_else(|| entry.get("_COMM")) // _COMM is often the executable name
        .or_else(|| entry.get("_SYSTEMD_UNIT"))
        .unwrap_or_else(|| "UnknownComponent".to_string());

    let hostname = entry.get("_HOSTNAME").unwrap_or_else(|| "localhost".to_string());
    // Journal entries might have other fields like PID (_PID), UID (_UID), etc.
    // which can be added to LogEntry if needed.

    Ok(LogEntry {
        timestamp: system_time,
        level: map_priority_to_loglevel(priority),
        message,
        component,
        hostname,
        // trace_id, span_id would need specific context not typically in journald
        trace_id: None,
        span_id: None,
    })
    // TODO: Unit test mapping of mock journal entries to LogEntry struct.
}

#[async_trait::async_trait]
impl LogHarvester for JournaldLogHarvester {
    /// Queries historical logs from journald based on filters, time range, and limit.
    ///
    /// Opens the system journal, seeks to the appropriate start time (if provided),
    /// and iterates through entries, applying filters for log level and keywords.
    ///
    /// # Arguments
    /// * `filter`: Optional `LogFilter` (level, keywords).
    /// * `time_range`: Optional `TimeRange` to define the query period.
    /// * `limit`: Optional maximum number of log entries to return.
    ///
    /// Returns a `Result` containing a `Vec<LogEntry>` or a `SystemError`.
    async fn query_logs(&self, filter: Option<LogFilter>, time_range: Option<TimeRange>, limit: Option<usize>) -> Result<Vec<LogEntry>, SystemError> {
        // TODO: Unit test filter application logic in query_logs.
        let mut journal = Journal::open_system()
            .map_err(|e: io::Error| SystemError::MetricCollectorError(format!("Failed to open journal: {}", e)))?;

        if let Some(ref tr) = time_range {
            // Seek to the start time of the range.
            let start_micros = tr.start_time.duration_since(UNIX_EPOCH).map_err(|_| SystemError::MetricCollectorError("Invalid start time".to_string()))?.as_micros();
            journal.seek(JournalSeek::RealtimeTimestamp(start_micros as u64))
                .map_err(|e| SystemError::MetricCollectorError(format!("Failed to seek to start time: {}", e)))?;
        } else {
             // Default: seek to head if no time range, or perhaps tail for recent logs?
             // For query, usually we want older logs first.
            journal.seek(JournalSeek::Head)
                .map_err(|e| SystemError::MetricCollectorError(format!("Failed to seek to head: {}", e)))?;
        }

        let mut logs = Vec::new();
        let mut count = 0;
        let max_entries = limit.unwrap_or(std::usize::MAX);

        // Iterate forwards from the seek point
        while let Some(entry_result) = journal.next_entry().map_err(|e| SystemError::MetricCollectorError(format!("Failed to read journal entry: {}", e)))? {
            if count >= max_entries {
                break;
            }

            let entry = entry_result; // sd_journal::JournalEntry

            if let Some(ref tr) = time_range {
                 let timestamp_us_str = entry.get("__REALTIME_TIMESTAMP")
                    .ok_or_else(|| SystemError::MetricCollectorError("Missing __REALTIME_TIMESTAMP for time filter".to_string()))?;
                let timestamp_us = timestamp_us_str.parse::<u64>()
                    .map_err(|e| SystemError::MetricCollectorError(format!("Failed to parse timestamp for time filter: {}", e)))?;
                let entry_time = UNIX_EPOCH + Duration::from_micros(timestamp_us);

                if entry_time > tr.end_time {
                    break; // Past the end of the time range
                }
                // Start time is handled by initial seek.
            }

            let log_entry = match journal_entry_to_log_entry(&entry) {
                Ok(le) => le,
                Err(e) => {
                    eprintln!("Error converting journal entry: {:?}", e); // Log and skip
                    continue;
                }
            };

            let mut matches_filter = true;
            if let Some(ref f) = filter {
                if let Some(min_level) = f.level {
                    if log_entry.level < min_level { // Assuming LogLevel implements Ord correctly
                        matches_filter = false;
                    }
                }
                if let Some(ref keywords) = f.keywords {
                    if !keywords.iter().any(|k| log_entry.message.contains(k) || log_entry.component.contains(k)) {
                        matches_filter = false;
                    }
                }
                // Note: component_filter from LogFilter is not directly used here,
                // but keywords can search in log_entry.component.
                // If a strict component match is needed, add:
                // if let Some(ref comp_filter) = f.component_filter {
                //    if !log_entry.component.starts_with(comp_filter) { // or .contains() or ==
                //        matches_filter = false;
                //    }
                // }
            }

            if matches_filter {
                logs.push(log_entry);
                count += 1;
            }
        }
        Ok(logs)
    }

    /// Streams live log entries from journald, applying optional filters.
    ///
    /// This implementation uses a basic polling mechanism: it seeks to the tail of the journal,
    /// reads any new entries, then waits for a short duration before polling again.
    /// A more advanced implementation might use `sd_journal_wait()` for more efficient blocking
    /// until new entries are available, integrated with an async runtime.
    ///
    /// # Arguments
    /// * `filter`: Optional `LogFilter` to apply to the streamed entries.
    ///
    /// Returns a `Result` containing a boxed `Stream` of `LogEntry` results, or a `SystemError`.
    async fn stream_logs(
        &self,
        filter: Option<LogFilter>,
    ) -> Result<Box<dyn Stream<Item = Result<LogEntry, SystemError>> + Send + Unpin>, SystemError> {
        // This implementation does not persist the cursor, so it always starts near the "live" end.

        let mut journal = Journal::open_system()
            .map_err(|e| SystemError::MetricCollectorError(format!("Failed to open journal for streaming: {}", e)))?;

        // Seek to the tail to get new messages.
        journal.seek(JournalSeek::Tail)
            .map_err(|e| SystemError::MetricCollectorError(format!("Failed to seek to tail for streaming: {}", e)))?;

        // Move to the previous entry to avoid re-sending the very last entry if the stream
        // is started immediately after a query. This behavior is similar to `journalctl -f`.
        if journal.previous_entry().map_err(|e| SystemError::MetricCollectorError(format!("Failed to seek to previous for streaming: {}", e)))?.is_none() {
            // If there was no previous entry (e.g., journal was empty or had only one),
            // re-seek to tail. This ensures we start at the correct position.
            journal.seek(JournalSeek::Tail)
                .map_err(|e| SystemError::MetricCollectorError(format!("Failed to re-seek to tail: {}", e)))?;
        }

        let s = stream! {
            loop {
                // sd_journal_wait can block, which is not ideal in an async context without specific handling.
                // For a true async stream, this would need to be integrated with an async event loop,
                // e.g., by spawning a blocking task for sd_journal_wait or using a crate that wraps it asynchronously.
                //
                // Simplified polling approach:
                // match journal.wait(Some(Duration::from_millis(100))) { // Wait with timeout
                //    Ok(true) => { // New entries
                //        // Process new entries
                //    }
                //    Ok(false) => { // Timeout, no new entries
                //        yield Ok(SystemError::MetricCollectorError("Polling timeout, retrying".to_string())); // This is not a LogEntry, stream type issue.
                //        continue;
                //    }
                //    Err(e) => {
                //        yield Err(SystemError::MetricCollectorError(format!("Journal wait error: {}", e)));
                //        break; // Or attempt to reopen journal
                //    }
                // }
                //
                // The above `wait` logic is complex to integrate directly into an `async_stream` block
                // without careful handling of blocking.
                //
                // Simplest polling: Read, then sleep.
                while let Some(entry_result) = journal.next_entry().map_err(|e| SystemError::MetricCollectorError(format!("Stream: Failed to read journal entry: {}", e)))? {
                     match journal_entry_to_log_entry(&entry_result) {
                        Ok(log_entry) => {
                            let mut matches_filter = true;
                            if let Some(ref f) = filter {
                                if let Some(min_level) = f.level {
                                    if log_entry.level < min_level {
                                        matches_filter = false;
                                    }
                                }
                                if let Some(ref keywords) = f.keywords {
                                    if !keywords.iter().any(|k| log_entry.message.contains(k) || log_entry.component.contains(k)) {
                                        matches_filter = false;
                                    }
                                }
                            }
                            if matches_filter {
                                yield Ok(log_entry);
                            }
                        }
                        Err(e) => {
                            yield Err(e); // Propagate conversion error
                        }
                    }
                }
                // After reading all current entries, wait a bit before polling again.
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };
        Ok(Box::pin(s))
    }
}
