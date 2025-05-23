#[cfg(test)]
mod tests {
    use crate::global_settings_management::errors::*;
    use crate::global_settings_management::paths::{SettingPath, AppearanceSettingPath, FontSettingPath};
    use novade_core::errors::CoreError; // For testing source conversion

    #[test]
    fn test_path_not_found_error_display() {
        let path = SettingPath::Appearance(AppearanceSettingPath::FontSettings(
            FontSettingPath::DefaultFontFamily,
        ));
        let error = GlobalSettingsError::PathNotFound { path: path.clone() };
        assert_eq!(
            error.to_string(),
            format!("Einstellungspfad nicht gefunden: {}", path)
        );
    }

    #[test]
    fn test_invalid_value_type_error_display() {
        let path = SettingPath::Appearance(AppearanceSettingPath::InterfaceScalingFactor);
        let error = GlobalSettingsError::InvalidValueType {
            path: path.clone(),
            expected_type: "f64".to_string(),
            actual_value_preview: "true".to_string(),
        };
        assert_eq!(
            error.to_string(),
            format!(
                "Ungültiger Wertetyp für Pfad {}: Erwartet 'f64', erhalten (Vorschau): 'true'",
                path
            )
        );
    }

    #[test]
    fn test_validation_error_display() {
        let path = SettingPath::Workspace(crate::global_settings_management::paths::WorkspaceSettingPath::DefaultWorkspaceCount);
        let error = GlobalSettingsError::ValidationError {
            path: path.clone(),
            reason: "Muss mindestens 1 sein".to_string(),
        };
        assert_eq!(
            error.to_string(),
            format!("Validierungsfehler für Pfad {}: Muss mindestens 1 sein", path)
        );
    }
    
    #[test]
    fn test_global_validation_failed_display() {
        let error = GlobalSettingsError::GlobalValidationFailed { reason: "Global consistency check failed".to_string() };
        assert_eq!(error.to_string(), "Validierungsfehler in globalen Einstellungen: Global consistency check failed");
    }

    #[test]
    fn test_serialization_error_display() {
        let path = SettingPath::Appearance(AppearanceSettingPath::ActiveThemeName);
        let error = GlobalSettingsError::SerializationError {
            path: path.clone(),
            source_message: "JSON error".to_string(),
        };
        assert_eq!(
            error.to_string(),
            format!("Serialisierungsfehler für Pfad {}: JSON error", path)
        );
    }

    #[test]
    fn test_deserialization_error_display() {
        let error = GlobalSettingsError::DeserializationError {
            source_message: "TOML parse error".to_string(),
        };
        assert_eq!(error.to_string(), "Deserialisierungsfehler: TOML parse error");
    }
    
    #[test]
    fn test_field_deserialization_error_display() {
        let path = SettingPath::Appearance(AppearanceSettingPath::ColorScheme);
        let error = GlobalSettingsError::FieldDeserializationError {
            path: path.clone(),
            source_message: "Invalid enum variant".to_string(),
        };
        assert_eq!(error.to_string(), format!("Deserialisierungsfehler für Pfad {}: Invalid enum variant", path));
    }


    #[test]
    fn test_persistence_error_display_with_source() {
        let core_err_str = "Underlying I/O error".to_string();
        let error = GlobalSettingsError::PersistenceError {
            operation: "save".to_string(),
            message: "Failed to write to disk".to_string(),
            source: Some(core_err_str.clone()),
        };
        // Note: thiserror's default Display for #[source] will append it.
        // The exact format depends on whether the source error's Display is multi-line.
        // For a simple string source, it might be "Persistenzfehler während Operation 'save': Failed to write to disk: Underlying I/O error"
        // Let's check if it contains the main parts.
        let display_str = error.to_string();
        assert!(display_str.contains("Persistenzfehler während Operation 'save': Failed to write to disk"));
        assert!(display_str.contains(&core_err_str));
    }
    
    #[test]
    fn test_persistence_error_display_without_source() {
        let error = GlobalSettingsError::PersistenceError {
            operation: "load".to_string(),
            message: "Configuration not found".to_string(),
            source: None,
        };
        assert_eq!(error.to_string(), "Persistenzfehler während Operation 'load': Configuration not found");
    }

    #[test]
    fn test_core_error_display() {
        let core_err_str = "Some core system failure".to_string();
        let error = GlobalSettingsError::CoreError(core_err_str.clone());
        assert_eq!(error.to_string(), format!("Core-Fehler: {}", core_err_str));
    }
    
    #[test]
    fn test_from_core_error_conversion() {
        let core_error = CoreError::ConfigFormatError("Bad format in core".to_string());
        let settings_error: GlobalSettingsError = core_error.clone().into(); // Assuming CoreError is Clone
        match settings_error {
            GlobalSettingsError::CoreError(msg) => assert_eq!(msg, core_error.to_string()),
            _ => panic!("Incorrect conversion from CoreError"),
        }
    }

    #[test]
    fn test_internal_error_display() {
        let error = GlobalSettingsError::InternalError("Something unexpected happened".to_string());
        assert_eq!(error.to_string(), "Interner Fehler: Something unexpected happened");
    }

    #[test]
    fn test_error_clonability() {
        let path = SettingPath::Appearance(AppearanceSettingPath::EnableAnimations);
        let original_error = GlobalSettingsError::ValidationError {
            path: path.clone(),
            reason: "Test reason".to_string(),
        };
        let cloned_error = original_error.clone();
        assert_eq!(original_error.to_string(), cloned_error.to_string());
        // Check a field for equality to be sure
        if let GlobalSettingsError::ValidationError { path: p1, reason: r1 } = original_error {
            if let GlobalSettingsError::ValidationError { path: p2, reason: r2 } = cloned_error {
                assert_eq!(p1, p2);
                assert_eq!(r1, r2);
            } else {
                panic!("Cloned error is not ValidationError variant");
            }
        } else {
            panic!("Original error is not ValidationError variant");
        }
    }
    
    #[test]
    fn test_persistence_error_helpers() {
        let op = "test_op";
        let msg = "test_message";
        
        let core_err = CoreError::NotFound("test_file_not_found".to_string());
        let err_with_source = GlobalSettingsError::persistence_error_with_core_source(op, msg, core_err.clone());
        match err_with_source {
            GlobalSettingsError::PersistenceError { operation, message, source } => {
                assert_eq!(operation, op);
                assert_eq!(message, msg);
                assert_eq!(source, Some(core_err.to_string()));
            }
            _ => panic!("Incorrect error from persistence_error_with_core_source"),
        }

        let err_without_source = GlobalSettingsError::persistence_error_without_source(op, msg);
         match err_without_source {
            GlobalSettingsError::PersistenceError { operation, message, source } => {
                assert_eq!(operation, op);
                assert_eq!(message, msg);
                assert_eq!(source, None);
            }
            _ => panic!("Incorrect error from persistence_error_without_source"),
        }
    }

    #[test]
    fn test_serde_error_helpers() {
        let path = SettingPath::Appearance(AppearanceSettingPath::AccentColorToken);
        // Mock a serde_json::Error (it's non-trivial to construct one directly for testing without actual serde_json usage)
        // For this test, we'll use a simple string that mimics what source_message would store.
        let json_error_str = "Unexpected token '}' at line 1 column 10".to_string();

        // Simulate a serde_json::Error by creating a DeserializationError from a string
        let de_err: Result<serde_json::Value, _> = serde_json::from_str("{invalid json");
        let actual_serde_error = de_err.err().unwrap();

        let serialization_err = GlobalSettingsError::serialization_error(path.clone(), actual_serde_error.clone());
         match serialization_err {
            GlobalSettingsError::SerializationError { path: p, source_message: sm } => {
                assert_eq!(p, path);
                assert_eq!(sm, actual_serde_error.to_string());
            }
            _ => panic!("Incorrect error from serialization_error helper"),
        }
        
        let deserialization_err = GlobalSettingsError::deserialization_error(actual_serde_error.clone());
        match deserialization_err {
            GlobalSettingsError::DeserializationError { source_message: sm } => {
                 assert_eq!(sm, actual_serde_error.to_string());
            }
            _ => panic!("Incorrect error from deserialization_error helper"),
        }

        let field_deserialization_err = GlobalSettingsError::field_deserialization_error(path.clone(), actual_serde_error.clone());
        match field_deserialization_err {
            GlobalSettingsError::FieldDeserializationError { path: p, source_message: sm } => {
                assert_eq!(p, path);
                assert_eq!(sm, actual_serde_error.to_string());
            }
            _ => panic!("Incorrect error from field_deserialization_error helper"),
        }
    }
}
