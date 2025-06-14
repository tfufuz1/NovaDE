// novade-domain/src/display_configuration/persistence_tests.rs
#![cfg(test)]
use super::persistence::{FileSystemDisplayPersistence, DisplayPersistence};
use novade_core::types::display::{Display, DisplayConfiguration, DisplayLayout, DisplayMode, DisplayConnector, DisplayStatus, PhysicalProperties};
use tempfile::NamedTempFile;
use std::io::Write;
use tokio::fs; // For async file operations if needed, though persistence ops are sync in example

// --- Helper function to create sample data ---
fn create_sample_display_mode(width: u32, height: u32, refresh: u32) -> DisplayMode {
    DisplayMode { width, height, refresh_rate: refresh }
}

fn create_sample_display(id: &str, name: &str) -> Display {
    Display {
        id: id.to_string(),
        name: name.to_string(),
        connector: DisplayConnector::HDMI,
        status: DisplayStatus::Connected,
        modes: vec![create_sample_display_mode(1920, 1080, 60000)],
        current_mode: Some(create_sample_display_mode(1920, 1080, 60000)),
        physical_properties: Some(PhysicalProperties { width_mm: 597, height_mm: 336 }),
        position_x: 0,
        position_y: 0,
        enabled: true,
    }
}

fn create_sample_display_config(displays: Vec<Display>, layout: DisplayLayout) -> DisplayConfiguration {
    DisplayConfiguration { displays, layout }
}

// --- Tests for FileSystemDisplayPersistence ---
#[tokio::test]
async fn test_save_and_load_config() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    let persistence = FileSystemDisplayPersistence::new(file_path.clone());
    let original_config = create_sample_display_config(
        vec![create_sample_display("DP-1", "Test Monitor")],
        DisplayLayout::Extended,
    );

    // Save the config
    persistence.save_config(&original_config).await.expect("Failed to save config");

    // Load the config
    let loaded_config = persistence.load_config().await.expect("Failed to load config");

    assert_eq!(original_config, loaded_config);

    // Clean up by closing the temp file, which also deletes it.
    // Explicitly drop if not handled by scope, though NamedTempFile handles this.
    drop(temp_file);
    assert!(!file_path.exists(), "Temp file should be deleted after drop");
}

#[tokio::test]
async fn test_load_non_existent_config() {
    // Path that should not exist
    let non_existent_path = std::env::temp_dir().join("non_existent_display_config.json");
    // Ensure it doesn't exist from a previous failed test run or similar
    if non_existent_path.exists() {
        std::fs::remove_file(&non_existent_path).expect("Could not clean up test file");
    }

    let persistence = FileSystemDisplayPersistence::new(non_existent_path.clone());
    let result = persistence.load_config().await;

    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            super::super::errors::DisplayConfigurationError::Persistence(msg) => {
                assert!(msg.contains("Config file not found"));
            }
            _ => panic!("Expected Persistence error, got {:?}", e),
        }
    }
    // Ensure file was not created
    assert!(!non_existent_path.exists());
}

#[tokio::test]
async fn test_load_corrupted_config() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();

    // Write invalid JSON to the file
    writeln!(temp_file, "{{ \"displays\": [\"invalid_json_format\" ]").expect("Failed to write corrupted data");
    temp_file.flush().expect("Failed to flush temp file");


    let persistence = FileSystemDisplayPersistence::new(file_path.clone());
    let result = persistence.load_config().await;

    assert!(result.is_err());
    if let Err(e) = result {
        match e {
            super::super::errors::DisplayConfigurationError::SerdeError(msg) => {
                // Specific error message might vary based on serde_json version and exact corruption
                assert!(msg.contains("expected value") || msg.contains("EOF") || msg.contains("invalid type"));
            }
            _ => panic!("Expected SerdeError, got {:?}", e),
        }
    }
    // temp_file is dropped here, deleting the file.
}

#[tokio::test]
async fn test_save_creates_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let nested_dir = temp_dir.path().join("some").join("nested");
    let file_path = nested_dir.join("display_config.json");

    // At this point, `nested_dir` does not exist.
    assert!(!nested_dir.exists());

    let persistence = FileSystemDisplayPersistence::new(file_path.clone());
    let config = create_sample_display_config(vec![], DisplayLayout::Single);

    persistence.save_config(&config).await.expect("Failed to save config, directory should be created");

    assert!(file_path.exists(), "Config file should have been created");
    assert!(nested_dir.exists(), "Nested directory should have been created by save_config");

    // temp_dir is dropped here, cleaning up the directory and its contents.
}
