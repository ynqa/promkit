/// Assert that two screens are equal, and if not, panic with a detailed diff of the two screens.
pub fn assert_screen_eq(expected: &[String], actual: &[String]) {
    if actual == expected {
        return;
    }

    panic!("{}", format_screen_diff(expected, actual));
}

/// Format a diff of two screens, showing the number of differing rows and the contents of each screen with differences highlighted.
fn format_screen_diff(expected: &[String], actual: &[String]) -> String {
    let total_rows = expected.len().max(actual.len());
    let differing_rows = (0..total_rows)
        .filter(|&row| expected.get(row) != actual.get(row))
        .count();

    let mut lines = vec![format!("screen mismatch ({differing_rows} differing rows)")];
    lines.push("expected:".to_string());
    lines.extend(format_screen(expected, total_rows));
    lines.push("actual:".to_string());
    lines.extend(format_screen(actual, total_rows));

    lines.join("\n")
}

/// Format a single line of the screen, replacing spaces with a visible character and marking missing lines.
fn format_screen_line(line: Option<&String>) -> String {
    match line {
        Some(line) => format!("|{}|", line.replace(' ', "·")),
        None => "<missing>".to_string(),
    }
}

/// Format an entire screen, prefixing each line with its row number and marking differences.
fn format_screen(lines: &[String], total_rows: usize) -> Vec<String> {
    (0..total_rows)
        .map(|row| format!("  r{row:02} {}", format_screen_line(lines.get(row))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format_screen_line {
        use super::*;

        #[test]
        fn replaces_spaces() {
            assert_eq!(format_screen_line(Some(&"a b c".to_string())), "|a·b·c|");
        }

        #[test]
        fn handles_empty_line() {
            assert_eq!(format_screen_line(Some(&"".to_string())), "||");
        }

        #[test]
        fn handles_missing_line() {
            assert_eq!(format_screen_line(None), "<missing>");
        }
    }

    mod format_screen {
        use super::*;

        #[test]
        fn formats_multiple_lines() {
            let lines = vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ];
            let formatted = format_screen(&lines, 5);
            assert_eq!(
                formatted,
                vec![
                    "  r00 |line·1|".to_string(),
                    "  r01 |line·2|".to_string(),
                    "  r02 |line·3|".to_string(),
                    "  r03 <missing>".to_string(),
                    "  r04 <missing>".to_string(),
                ]
            );
        }
    }
}
