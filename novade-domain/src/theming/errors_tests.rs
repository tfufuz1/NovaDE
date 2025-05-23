// This is a new file for error tests.
// Alternatively, these could be in `theming::tests::errors_tests` or similar.
// For simplicity, placing it here and will `#[path = ...]` or `mod errors_tests;` in `theming::mod.rs` (or `lib.rs` if needed for tests).

#[cfg(test)]
mod tests {
    use crate::theming::errors::ThemingError;
    use crate::theming::types::{ThemeIdentifier, TokenIdentifier};

    #[test]
    fn theming_error_is_cloneable() {
        let error1 = ThemingError::ThemeNotFound {
            theme_id: ThemeIdentifier::new("test-theme"),
        };
        let error2 = error1.clone();
        assert_eq!(format!("{}", error1), format!("{}", error2));

        let token_error = ThemingError::TokenNotFound {
            token_id: TokenIdentifier::new("test-token"),
        };
        let token_error_clone = token_error.clone();
        assert_eq!(format!("{}", token_error), format!("{}", token_error_clone));

        let cyclic_error = ThemingError::CyclicTokenReference {
            token_id: TokenIdentifier::new("cyclic-token"),
            path: vec![TokenIdentifier::new("path-part-1")],
        };
        let cyclic_error_clone = cyclic_error.clone();
        assert_eq!(cyclic_error.token_id, cyclic_error_clone.token_id);
        assert_eq!(cyclic_error.path, cyclic_error_clone.path);


        let parse_error = ThemingError::TokenFileParseError {
            file_path: "some/file.json".to_string(),
            source_message: "some serde error".to_string(),
        };
        let parse_error_clone = parse_error.clone();
        match (parse_error, parse_error_clone) {
            (
                ThemingError::TokenFileParseError { file_path: fp1, source_message: sm1 },
                ThemingError::TokenFileParseError { file_path: fp2, source_message: sm2 },
            ) => {
                assert_eq!(fp1, fp2);
                assert_eq!(sm1, sm2);
            }
            _ => panic!("Errors did not match after cloning"),
        }
    }

    #[test]
    fn theming_error_invalid_value_helper() {
        let token_id = TokenIdentifier::new("my-token");
        let message = "This value is not good.";
        let error = ThemingError::invalid_value(token_id.clone(), message);
        match error {
            ThemingError::InvalidTokenValue { token_id: tid, message: msg } => {
                assert_eq!(tid, token_id);
                assert_eq!(msg, message);
            }
            _ => panic!("Incorrect error type from helper"),
        }
    }
}
