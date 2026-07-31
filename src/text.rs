//! Shared text helpers.

/// Truncate to `max` characters, ending with an ellipsis when cut. Counts
/// chars, never bytes: byte slicing panics on a multi-byte boundary, and the
/// inputs here (API error messages, revisions, container names) can carry
/// arbitrary UTF-8.
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    match max {
        0 => String::new(),
        _ => {
            let mut t: String = s.chars().take(max - 1).collect();
            t.push('…');
            t
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_strings_pass_through() {
        assert_eq!(ellipsize("abc", 3), "abc");
        assert_eq!(ellipsize("", 5), "");
    }

    #[test]
    fn long_strings_end_with_ellipsis_at_max_chars() {
        assert_eq!(ellipsize("abcdef", 4), "abc…");
        assert_eq!(ellipsize("abcdef", 4).chars().count(), 4);
    }

    #[test]
    fn zero_max_is_empty() {
        assert_eq!(ellipsize("abc", 0), "");
    }

    #[test]
    fn multibyte_input_never_panics() {
        // Regression: a byte-sliced truncation panicked when byte 59 fell
        // inside a multi-byte sequence.
        let s = "é".repeat(80);
        assert_eq!(ellipsize(&s, 60).chars().count(), 60);
        assert_eq!(ellipsize("αβγδε", 3), "αβ…");
    }
}
