//! # Disk Metrics Collector
//!
//! This module is responsible for collecting disk I/O activity and disk space usage statistics
//! on Linux systems. It utilizes `/proc/diskstats` for activity and `/proc/mounts` combined
//! with `statvfs` for space usage.

use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncReadExt}; // BufReader not strictly needed
use tokio::time::{sleep, Duration};
use novade_core::types::system_health::{DiskActivityMetrics, DiskSpaceMetrics};
use crate::error::SystemError;
use crate::system_health_collectors::DiskMetricsCollector;
use std::collections::HashMap;
// Ensure nix is present in Cargo.toml with "fs" and "mount" features for statvfs
use nix::sys::statvfs;
use std::path::Path;

/// A collector for disk metrics on Linux systems.
///
/// Implements the `DiskMetricsCollector` trait.
/// - For disk activity: Parses `/proc/diskstats` at two intervals to calculate I/O rates
///   (reads/s, writes/s, bytes/s, busy time) for physical block devices.
/// - For disk space: Parses `/proc/mounts` to find mounted filesystems and uses `statvfs`
///   (via the `nix` crate) to get total, used, free, and available space for each.
pub struct LinuxDiskMetricsCollector;

/// Holds raw statistics for a single block device read from `/proc/diskstats`.
#[derive(Clone, Debug)]
struct DiskStats {
    /// The name of the disk device (e.g., "sda", "nvme0n1").
    device_name: String,
    /// Number of reads completed.
    reads_completed: u64,
    /// Number of sectors read.
    sectors_read: u64,
    /// Time spent reading (in milliseconds).
    time_spent_reading_ms: u64,
    /// Number of writes completed.
    writes_completed: u64,
    /// Number of sectors written.
    sectors_written: u64,
    /// Time spent writing (in milliseconds).
    time_spent_writing_ms: u64,
}

/// Standard sector size in bytes, commonly used for calculations involving sector counts.
const SECTOR_SIZE_BYTES: u64 = 512;

/// Asynchronously reads and parses `/proc/diskstats`.
///
/// Filters out loop devices and RAM disks, focusing on physical or main virtual block devices.
/// Each relevant line is parsed into a `DiskStats` struct.
/// See kernel documentation for `/proc/diskstats` for field definitions.
///
/// Returns a `Result` containing a `Vec<DiskStats>` or a `SystemError`.
async fn read_proc_diskstats() -> Result<Vec<DiskStats>, SystemError> {
    let mut file = File::open("/proc/diskstats").await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to open /proc/diskstats: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to read /proc/diskstats: {}", e)))?;

    let mut stats = Vec::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 { // Minimum number of fields for disk stats
            continue;
        }
        // Filter out loop devices and other non-physical/virtual main block devices
        let device_name = parts[2].to_string();
        if device_name.starts_with("loop") || device_name.starts_with("ram") || device_name.starts_with("sr") {
            continue;
        }

        // Fields based on kernel documentation for /proc/diskstats:
        // 3 - reads completed successfully
        // 5 - sectors read successfully
        // 6 - time spent reading (ms)
        // 7 - writes completed successfully
        // 9 - sectors written successfully
        // 10 - time spent writing (ms)
        // Note: indices are 0-based for `parts` vector.
        // Original documentation field numbers are 1-based.
        // field 4 -> parts[3]
        // field 6 -> parts[5]
        // field 7 -> parts[6]
        // field 8 -> parts[7]
        // field 10 -> parts[9]
        // field 11 -> parts[10]
        stats.push(DiskStats {
            device_name,
            reads_completed: parts[3].parse().unwrap_or(0),
            sectors_read: parts[5].parse().unwrap_or(0),
            time_spent_reading_ms: parts[6].parse().unwrap_or(0),
            writes_completed: parts[7].parse().unwrap_or(0),
            sectors_written: parts[9].parse().unwrap_or(0),
            time_spent_writing_ms: parts[10].parse().unwrap_or(0),
        });
    }
    Ok(stats)
    // TODO: Unit test parsing of /proc/diskstats mock data and I/O rate calculation.
}

/// Asynchronously reads `/proc/mounts` to get a list of currently mounted filesystems.
///
/// Filters for common filesystem types (ext*, xfs, btrfs, vfat, ntfs, fuseblk) to exclude
/// most virtual or pseudo filesystems.
///
/// Returns a `Result` containing a `Vec<String>` of mount points, or a `SystemError`.
async fn read_proc_mounts() -> Result<Vec<String>, SystemError> {
    // TODO: Unit test parsing of /proc/mounts mock data for DiskSpaceMetrics.
    let mut file = File::open("/proc/mounts").await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to open /proc/mounts: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await
        .map_err(|e| SystemError::MetricCollectorError(format!("Failed to read /proc/mounts: {}", e)))?;

    let mut mount_points = Vec::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // We are interested in the mount point, which is the second field.
            // We also filter for common filesystem types to avoid virtual/pseudo filesystems
            // although statvfs should handle most cases correctly.
            let fs_type = parts[2];
            if fs_type.starts_with("ext") || fs_type.starts_with("xfs") || fs_type.starts_with("btrfs") ||
               fs_type.starts_with("vfat") || fs_type.starts_with("ntfs") || fs_type.starts_with("fuseblk") {
                mount_points.push(parts[1].to_string());
            }
        }
    }
    Ok(mount_points)
}


#[async_trait::async_trait]
impl DiskMetricsCollector for LinuxDiskMetricsCollector {
    /// Asynchronously collects disk I/O activity metrics.
    ///
    /// Reads `/proc/diskstats` twice with a delay, calculates differences in I/O counters
    /// (reads, writes, sectors, time spent) to determine rates like IOPS, throughput (bytes/sec),
    /// and busy time percentage for each relevant block device.
    ///
    /// Returns a `Result` with a `Vec<DiskActivityMetrics>` or a `SystemError`.
    async fn collect_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, SystemError> {
        let stats1_map: HashMap<String, DiskStats> = read_proc_diskstats().await?
            .into_iter()
            .map(|s| (s.device_name.clone(), s))
            .collect();

        // A short delay is essential for calculating rates from cumulative counters.
        sleep(Duration::from_millis(500)).await;

        let stats2_list = read_proc_diskstats().await?;
        let mut activity_metrics = Vec::new();

        for stats2 in stats2_list {
            if let Some(stats1) = stats1_map.get(&stats2.device_name) {
                let duration_s = 0.5; // Corresponds to the sleep duration above.

                let reads_per_second = (stats2.reads_completed - stats1.reads_completed) as f64 / duration_s;
                let writes_per_second = (stats2.writes_completed - stats1.writes_completed) as f64 / duration_s;
                let read_bytes_per_second = ((stats2.sectors_read - stats1.sectors_read) * SECTOR_SIZE_BYTES) as f64 / duration_s;
                let written_bytes_per_second = ((stats2.sectors_written - stats1.sectors_written) * SECTOR_SIZE_BYTES) as f64 / duration_s;

                // Calculate busy time percentage. Time spent is in ms.
                let read_busy_time_percent = if stats2.reads_completed > stats1.reads_completed {
                    ((stats2.time_spent_reading_ms - stats1.time_spent_reading_ms) as f64 / (duration_s * 1000.0)) * 100.0
                } else { 0.0 };
                let write_busy_time_percent = if stats2.writes_completed > stats1.writes_completed {
                    ((stats2.time_spent_writing_ms - stats1.time_spent_writing_ms) as f64 / (duration_s * 1000.0)) * 100.0
                } else { 0.0 };

                activity_metrics.push(DiskActivityMetrics {
                    device_name: stats2.device_name.clone(),
                    reads_per_second: reads_per_second as f32,
                    writes_per_second: writes_per_second as f32,
                    read_bytes_per_second: read_bytes_per_second as u64,
                    written_bytes_per_second: written_bytes_per_second as u64,
                    read_busy_time_percent: read_busy_time_percent.min(100.0) as f32, // Cap at 100%
                    write_busy_time_percent: write_busy_time_percent.min(100.0) as f32, // Cap at 100%
                });
            }
        }
        Ok(activity_metrics)
    }

    /// Asynchronously collects disk space usage metrics.
    ///
    /// Reads `/proc/mounts` to identify mounted filesystems, then uses `statvfs`
    /// for each to retrieve total, used, free, and available disk space in bytes.
    /// `filesystem_type` in `DiskSpaceMetrics` is not populated by this collector
    /// as `statvfs` does not directly provide it.
    ///
    /// Returns a `Result` with a `Vec<DiskSpaceMetrics>` or a `SystemError`.
    async fn collect_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, SystemError> {
        let mount_points = read_proc_mounts().await?;
        let mut space_metrics = Vec::new();

        for mount_point_str in mount_points {
            let path = Path::new(&mount_point_str);
            match statvfs::statvfs(path) {
                Ok(stat) => {
                    let total_bytes = stat.blocks() * stat.fragment_size();
                    // `blocks_available` is free blocks available to non-superuser.
                    let free_bytes_for_user = stat.blocks_available() * stat.fragment_size();
                    // `blocks_free` is total free blocks.
                    let free_bytes_total = stat.blocks_free() * stat.fragment_size();
                    let used_bytes = total_bytes - free_bytes_total;

                    space_metrics.push(DiskSpaceMetrics {
                        mount_point: mount_point_str.clone(),
                        // statvfs doesn't provide filesystem type; this would need to be sourced
                        // from /proc/mounts's third field if desired.
                        filesystem_type: String::new(),
                        total_bytes,
                        used_bytes,
                        free_bytes: free_bytes_total,
                        available_bytes: free_bytes_for_user,
                    });
                }
                Err(e) => {
                    // Log error or push a specific error metric? For now, skip problematic mounts.
                    eprintln!("Failed to statvfs for mount point {}: {}", mount_point_str, e);
                     // Or return Err(SystemError::MetricCollectorError(format!("Failed to statvfs for {}: {}", mount_point_str, e)))
                }
            }
        }
        Ok(space_metrics)
    }
}
