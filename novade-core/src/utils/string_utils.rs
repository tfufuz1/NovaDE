//! String utilities for the NovaDE core layer.
//!
//! This module provides string-related utilities used throughout the
//! NovaDE desktop environment.

/// Truncates a string to the specified maximum length.
///
/// If the string is longer than the maximum length, it is truncated
/// and an ellipsis is appended.
///
/// # Arguments
///
/// * `s` - The string to truncate
/// * `max_len` - The maximum length
///
/// # Returns
///
/// The truncated string.
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Ensure we have room for the ellipsis
        let truncate_len = if max_len > 3 { max_len - 3 } else { 0 };
        let mut result = s.chars().take(truncate_len).collect::<String>();
        result.push_str("...");
        result
    }
}

/// Formats a byte count as a human-readable string.
///
/// # Arguments
///
/// * `bytes` - The number of bytes
/// * `precision` - The number of decimal places to include
///
/// # Returns
///
/// A human-readable string representation of the byte count.
pub fn format_bytes(bytes: u64, precision: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes = bytes as f64;
    let base = 1024_f64;
    
    // Calculate the appropriate unit
    let exp = (bytes.ln() / base.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    
    // Format the number with the specified precision
    let value = bytes / base.powi(exp as i32);
    let format_str = format!("{{:.{}f}} {{}}", precision);
    
    format!(format_str, value, UNITS[exp])
}

/// Converts a string to snake_case.
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The converted string.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    
    for (i, c) in s.char_indices() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }
    
    result
}

/// Converts a string to camelCase.
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The converted string.
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut first_char = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next || (!first_char && c.is_uppercase()) {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            if first_char {
                result.push(c.to_lowercase().next().unwrap());
                first_char = false;
            } else {
                result.push(c);
            }
            capitalize_next = false;
        }
    }
    
    result
}

/// Converts a string to PascalCase.
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The converted string.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Converts a string to kebab-case.
///
/// # Arguments
///
/// * `s` - The string to convert
///
/// # Returns
///
/// The converted string.
pub fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    
    for (i, c) in s.char_indices() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else if c == '_' || c == ' ' {
            result.push('-');
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello, world!", 20), "Hello, world!");
        assert_eq!(truncate_string("Hello, world!", 5), "He...");
        assert_eq!(truncate_string("Hello", 5), "Hello");
        assert_eq!(truncate_string("Hello", 3), "...");
        assert_eq!(truncate_string("Hello", 0), "...");
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0, 2), "0 B");
        assert_eq!(format_bytes(1023, 2), "1023.00 B");
        assert_eq!(format_bytes(1024, 2), "1.00 KB");
        assert_eq!(format_bytes(1536, 2), "1.50 KB");
        assert_eq!(format_bytes(1048576, 2), "1.00 MB");
        assert_eq!(format_bytes(1073741824, 2), "1.00 GB");
        assert_eq!(format_bytes(1099511627776, 2), "1.00 TB");
        
        // Test different precision
        assert_eq!(format_bytes(1536, 0), "2 KB");
        assert_eq!(format_bytes(1536, 1), "1.5 KB");
        assert_eq!(format_bytes(1536, 3), "1.500 KB");
    }
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello_world"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello-world");
        assert_eq!(to_snake_case("HELLO_WORLD"), "h_e_l_l_o__w_o_r_l_d");
        assert_eq!(to_snake_case("HTTPRequest"), "http_request");
    }
    
    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(to_camel_case("helloWorld"), "helloWorld");
        assert_eq!(to_camel_case("HELLO_WORLD"), "helloWorld");
        assert_eq!(to_camel_case("HTTP_REQUEST"), "httpRequest");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("HELLO_WORLD"), "HelloWorld");
        assert_eq!(to_pascal_case("HTTP_REQUEST"), "HttpRequest");
    }
    
    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("hello-world"), "hello-world");
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(to_kebab_case("HELLO_WORLD"), "hello-world");
        assert_eq!(to_kebab_case("HTTP_REQUEST"), "http-request");
    }
}
