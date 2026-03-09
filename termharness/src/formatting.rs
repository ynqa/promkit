pub fn format_screen_diff(expected: &[String], actual: &[String]) -> String {
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

pub fn format_screen_line(line: Option<&String>) -> String {
    match line {
        Some(line) => format!("|{}|", line.replace(' ', "·")),
        None => "<missing>".to_string(),
    }
}

pub fn format_screen(lines: &[String], total_rows: usize) -> Vec<String> {
    (0..total_rows)
        .map(|row| format!("  r{row:02} {}", format_screen_line(lines.get(row))))
        .collect()
}
