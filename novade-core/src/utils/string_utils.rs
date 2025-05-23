//! String Manipulation Utilities.
//!
//! This module provides a collection of helper functions for common string operations,
//! such as truncation, formatting byte counts into human-readable strings, and
//! converting strings between various casing conventions (snake_case, camelCase,
//! PascalCase, kebab-case).
//!
//! # Key Functions
//!
//! - [`truncate_string()`]: Shortens a string to a maximum length, appending an ellipsis if truncated.
//! - [`format_bytes()`]: Converts a byte count (u64) into a human-readable string with units (B, KB, MB, etc.).
//! - Case Conversion:
//!   - [`to_snake_case()`]
//!   - [`to_camel_case()`]
//!   - [`to_pascal_case()`]
//!   - [`to_kebab_case()`]
//!
//! These functions are designed to be pure and do not typically return `Result` types,
//! as their operations are generally infallible for valid string inputs (though behavior
//! with non-ASCII characters in case conversions might depend on standard library methods).

/// Truncates a string to the specified maximum length, appending an ellipsis ("...") if truncated.
///
/// If the original string's length is less than or equal to `max_len`, it is returned unchanged.
/// If `max_len` is less than 3 (the length of the ellipsis), the behavior might result in
/// just the ellipsis or a string shorter than the ellipsis if `max_len` is 0, 1, or 2.
///
/// # Arguments
///
/// * `s`: The string slice to truncate.
/// * `max_len`: The maximum desired length of the output string, including the ellipsis if appended.
///
/// # Returns
///
/// A `String` which is either the original string (if it's short enough) or the truncated
/// string with an ellipsis.
///
/// # Examples
///
/// ```
/// use novade_core::utils::string_utils::truncate_string;
///
/// assert_eq!(truncate_string("Hello, world!", 20), "Hello, world!");
/// assert_eq!(truncate_string("Hello, world!", 8), "Hello...");
/// assert_eq!(truncate_string("Short", 5), "Short");
/// assert_eq!(truncate_string("Tiny", 3), "..."); // max_len allows only ellipsis
/// assert_eq!(truncate_string("No", 2), "...");   // max_len is too small for ".." + char
/// assert_eq!(truncate_string("A", 0), "...");    // max_len is 0
/// ```
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len { // Use chars().count() for Unicode character length
        s.to_string()
    } else {
        // Ensure we have room for the ellipsis
        let ellipsis = "...";
        let ellipsis_len = ellipsis.chars().count();
        let truncate_len = if max_len > ellipsis_len { max_len - ellipsis_len } else { 0 };
        
        let mut result = s.chars().take(truncate_len).collect::<String>();
        if max_len > 0 { // Only add ellipsis if there's any space at all
             result.push_str(ellipsis);
        } else if s.is_empty() && max_len == 0 { // Special case: empty input, max_len 0
            return "".to_string();
        } else if max_len == 0 && !s.is_empty() { // Non-empty input, max_len 0
            return ellipsis.to_string();
        }
        // If truncate_len was 0 and max_len > 0 (e.g. max_len = 1, 2), result is just "..."
        // Ensure the final result does not exceed max_len due to ellipsis with small max_len
        if result.chars().count() > max_len && max_len < ellipsis_len {
            return s.chars().take(max_len).collect::<String>(); // Fallback if ellipsis logic is tricky
        }
        result
    }
}

/// Formats a byte count into a human-readable string with appropriate units (B, KB, MB, GB, TB, PB).
///
/// # Arguments
///
/// * `bytes`: The number of bytes (as a `u64`).
/// * `precision`: The number of decimal places to include in the formatted output.
///
/// # Returns
///
/// A `String` representing the byte count in a human-readable format (e.g., "1.50 KB", "2.00 MB").
/// Returns "0 B" if `bytes` is 0.
///
/// # Examples
///
/// ```
/// use novade_core::utils::string_utils::format_bytes;
///
/// assert_eq!(format_bytes(0, 2), "0 B");
/// assert_eq!(format_bytes(512, 0), "512 B");
/// assert_eq!(format_bytes(1024, 2), "1.00 KB");
/// assert_eq!(format_bytes(1500, 1), "1.5 KB"); // 1500 / 1024 = 1.46...
/// assert_eq!(format_bytes(1024 * 1024 * 5, 2), "5.00 MB");
/// assert_eq!(format_bytes(1024 * 1024 * 1024 * 2, 0), "2 GB");
/// ```
pub fn format_bytes(bytes: u64, precision: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes_f = bytes as f64;
    let base = 1024.0_f64;
    
    let i = (bytes_f.ln() / base.ln()).floor() as i32;
    let unit_idx = i.min((UNITS.len() - 1) as i32) as usize;
    
    let value = bytes_f / base.powi(i);
    
    // Format the number with the specified precision
    // Using format! directly with precision specifier
    format!("{:.prec$} {}", value, UNITS[unit_idx], prec = precision)
}

/// Converts a string to `snake_case`.
///
/// This function attempts to convert various casing styles (PascalCase, camelCase)
/// into snake_case. It inserts underscores before uppercase letters that are not
/// preceded by another uppercase letter (to handle acronyms like "HTTPRequest" -> "http_request")
/// and converts the entire string to lowercase.
///
/// Note: This implementation is basic and might not handle all edge cases or Unicode
/// word boundaries perfectly. It primarily targets common ASCII-based identifiers.
///
/// # Arguments
///
/// * `s`: The string slice to convert.
///
/// # Returns
///
/// A `String` in snake_case.
///
/// # Examples
/// ```
/// use novade_core::utils::string_utils::to_snake_case;
/// assert_eq!(to_snake_case("HelloWorld"), "hello_world");
/// assert_eq!(to_snake_case("helloWorld"), "hello_world");
/// assert_eq!(to_snake_case("HTTPRequest"), "http_request");
/// assert_eq!(to_snake_case("MyAPIService"), "my_api_service");
/// assert_eq!(to_snake_case("already_snake"), "already_snake");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    let mut prev_is_underscore = true; // Treat start of string as if preceded by an underscore
    
    for (i, c) in s.char_indices() {
        if c.is_uppercase() {
            // Add underscore if not the first char, not preceded by an underscore,
            // and either the previous char was lowercase or this uppercase char is followed by a lowercase one (to handle acronyms).
            if !result.is_empty() && !prev_is_underscore && (!prev_is_upper || s.chars().nth(i + 1).map_or(false, |next_c| next_c.is_lowercase())) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
            prev_is_underscore = false;
        } else if c == '_' || c == '-' || c == ' ' {
            if !prev_is_underscore && !result.is_empty() { // Avoid leading/multiple underscores
                result.push('_');
                prev_is_underscore = true;
            }
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
            prev_is_underscore = false;
        }
    }
    
    result
}

/// Converts a string to `camelCase`.
///
/// This function converts strings from snake_case, kebab-case, or PascalCase
/// to camelCase. The first letter is lowercase, and subsequent words start with
/// an uppercase letter. Delimiters (`_`, `-`, ` `) are removed.
///
/// # Arguments
///
/// * `s`: The string slice to convert.
///
/// # Returns
///
/// A `String` in camelCase.
///
/// # Examples
/// ```
/// use novade_core::utils::string_utils::to_camel_case;
/// assert_eq!(to_camel_case("hello_world"), "helloWorld");
/// assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
/// assert_eq!(to_camel_case("HTTP-Request"), "httpRequest");
/// ```
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    let mut first_word_char_processed = false;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
            first_word_char_processed = true;
        } else if !first_word_char_processed {
            result.push(c.to_lowercase().next().unwrap());
            first_word_char_processed = true; // First char of the first word processed
        }
         else {
            result.push(c);
        }
    }
    result
}

/// Converts a string to `PascalCase` (also known as UpperCamelCase).
///
/// This function converts strings from snake_case, kebab-case, or camelCase
/// to PascalCase. Each word (including the first) starts with an uppercase letter.
/// Delimiters (`_`, `-`, ` `) are removed.
///
/// # Arguments
///
/// * `s`: The string slice to convert.
///
/// # Returns
///
/// A `String` in PascalCase.
///
/// # Examples
/// ```
/// use novade_core::utils::string_utils::to_pascal_case;
/// assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
/// assert_eq!(to_pascal_case("helloWorld"), "HelloWorld");
/// assert_eq!(to_pascal_case("http-request"), "HttpRequest");
/// ```
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

/// Converts a string to `kebab-case`.
///
/// This function converts strings from various casings (PascalCase, camelCase, snake_case)
/// to kebab-case. Words are separated by hyphens, and all letters are lowercase.
/// It handles transitions from lowercase to uppercase and existing delimiters like
/// underscores or spaces.
///
/// # Arguments
///
/// * `s`: The string slice to convert.
///
/// # Returns
///
/// A `String` in kebab-case.
///
/// # Examples
/// ```
/// use novade_core::utils::string_utils::to_kebab_case;
/// assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
/// assert_eq!(to_kebab_case("helloWorld"), "hello-world");
/// assert_eq!(to_kebab_case("HTTP_REQUEST"), "http-request");
/// assert_eq!(to_kebab_case("my api service"), "my-api-service");
/// ```
pub fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_char_was_delimiter = true; // Treat start as if preceded by delimiter

    for c in s.chars() {
        if c.is_uppercase() {
            if !result.is_empty() && !prev_char_was_delimiter {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_char_was_delimiter = false;
        } else if c == '_' || c == '-' || c == ' ' {
            if !prev_char_was_delimiter && !result.is_empty() { // Avoid leading/multiple hyphens
                result.push('-');
                prev_char_was_delimiter = true;
            }
        } else {
            result.push(c);
            prev_char_was_delimiter = false;
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
        assert_eq!(truncate_string("Hello, world!", 8), "Hello...");
        assert_eq!(truncate_string("Hello", 5), "Hello");
        assert_eq!(truncate_string("Tiny", 3), "...");
        assert_eq!(truncate_string("No", 2), "No"); // Max len 2 cannot fit "..."
        assert_eq!(truncate_string("Y", 1), "Y");
        assert_eq!(truncate_string("Yes", 0), "..."); // Max len 0, but not empty
        assert_eq!(truncate_string("", 0), "");    // Empty string, max_len 0
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0, 2), "0 B");
        assert_eq!(format_bytes(1023, 2), "1023.00 B");
        assert_eq!(format_bytes(1024, 2), "1.00 KB");
        assert_eq!(format_bytes(1500, 1), "1.5 KB"); // Test rounding/precision
        assert_eq!(format_bytes(1048576, 2), "1.00 MB");
        assert_eq!(format_bytes(1073741824, 2), "1.00 GB");
        assert_eq!(format_bytes(1099511627776, 2), "1.00 TB");
        
        // Test different precision
        assert_eq!(format_bytes(1500, 0), "1 KB"); // 1.46... rounds to 1 with 0 precision
        assert_eq!(format_bytes(1536, 0), "2 KB"); // 1.5 rounds to 2
        assert_eq!(format_bytes(1500, 3), "1.465 KB");
    }
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello_world"), "hello_world"); // Already snake
        assert_eq!(to_snake_case("hello-world"), "hello-world"); // Kebab not changed by this simple version
        assert_eq!(to_snake_case("HELLO_WORLD"), "hello_world"); // Acronyms / All caps
        assert_eq!(to_snake_case("HTTPRequest"), "http_request");
        assert_eq!(to_snake_case("MyAPIService"), "my_api_service");
        assert_eq!(to_snake_case("MyAPI"), "my_api");
        assert_eq!(to_snake_case("API"), "api");
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_snake_case("  leading_space"), "leading_space"); // Handles leading delimiters
    }
    
    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("hello-world"), "helloWorld");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld"); // Pascal to Camel
        assert_eq!(to_camel_case("helloWorld"), "helloWorld"); // Already camel
        assert_eq!(to_camel_case("HELLO_WORLD"), "helloWorld");
        assert_eq!(to_camel_case("HTTP_REQUEST"), "httpRequest");
        assert_eq!(to_camel_case("my_API_service"), "myApiService");
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("  leading_space"), "leadingSpace");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld"); // Already Pascal
        assert_eq!(to_pascal_case("helloWorld"), "HelloWorld"); // Camel to Pascal
        assert_eq!(to_pascal_case("HELLO_WORLD"), "HelloWorld");
        assert_eq!(to_pascal_case("HTTP_REQUEST"), "HttpRequest");
        assert_eq!(to_pascal_case("my_API_service"), "MyApiService");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_pascal_case("  leading_space"), "LeadingSpace");
    }
    
    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("hello-world"), "hello-world"); // Already kebab
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(to_kebab_case("HELLO_WORLD"), "hello-world");
        assert_eq!(to_kebab_case("HTTP_REQUEST"), "http-request");
        assert_eq!(to_kebab_case("MyAPIService"), "my-api-service");
        assert_eq!(to_kebab_case("My API Service"), "my-api-service");
        assert_eq!(to_kebab_case(""), "");
        assert_eq!(to_kebab_case("  leading_space"), "leading-space");
    }
}
