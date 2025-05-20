//! Default configuration values for the NovaDE core layer.
//!
//! This module provides default values for configuration settings
//! used throughout the NovaDE desktop environment.

use std::thread;

/// Default log level.
pub fn default_log_level() -> String {
    "info".to_string()
}

/// Default log file path.
pub fn default_log_file() -> String {
    "novade.log".to_string()
}

/// Default setting for logging to console.
pub fn default_log_to_console() -> bool {
    true
}

/// Default application name.
pub fn default_app_name() -> String {
    "novade".to_string()
}

/// Default application version.
pub fn default_app_version() -> String {
    "0.1.0".to_string()
}

/// Default data directory.
pub fn default_data_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.local/share/novade", home)
}

/// Default cache directory.
pub fn default_cache_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.cache/novade", home)
}

/// Default config directory.
pub fn default_config_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.config/novade", home)
}

/// Default number of worker threads.
pub fn default_worker_threads() -> usize {
    let num_cpus = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);
    
    // Use at most 75% of available CPUs, but at least 1
    std::cmp::max(1, num_cpus * 3 / 4)
}

/// Default setting for hardware acceleration.
pub fn default_use_hardware_acceleration() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_log_level() {
        assert_eq!(default_log_level(), "info");
    }
    
    #[test]
    fn test_default_log_file() {
        assert_eq!(default_log_file(), "novade.log");
    }
    
    #[test]
    fn test_default_log_to_console() {
        assert!(default_log_to_console());
    }
    
    #[test]
    fn test_default_app_name() {
        assert_eq!(default_app_name(), "novade");
    }
    
    #[test]
    fn test_default_app_version() {
        assert_eq!(default_app_version(), "0.1.0");
    }
    
    #[test]
    fn test_default_data_dir() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(default_data_dir(), format!("{}/.local/share/novade", home));
    }
    
    #[test]
    fn test_default_cache_dir() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(default_cache_dir(), format!("{}/.cache/novade", home));
    }
    
    #[test]
    fn test_default_config_dir() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        assert_eq!(default_config_dir(), format!("{}/.config/novade", home));
    }
    
    #[test]
    fn test_default_worker_threads() {
        let num_cpus = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        
        let expected = std::cmp::max(1, num_cpus * 3 / 4);
        assert_eq!(default_worker_threads(), expected);
    }
    
    #[test]
    fn test_default_use_hardware_acceleration() {
        assert!(default_use_hardware_acceleration());
    }
}
