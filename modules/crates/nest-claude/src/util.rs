//! Small helpers shared across request-building modules.

/// Percent-encodes a query-string value, leaving unreserved characters as-is.
pub(crate) fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_reserved_characters() {
        assert_eq!(percent_encode("page_abc="), "page_abc%3D");
        assert_eq!(percent_encode("[gte]"), "%5Bgte%5D");
    }

    #[test]
    fn leaves_unreserved_characters_untouched() {
        assert_eq!(percent_encode("abc-123_XYZ.~"), "abc-123_XYZ.~");
    }
}
