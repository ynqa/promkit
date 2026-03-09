use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Screen {
    cols: u16,
    rows: u16,
    lines: Vec<Option<String>>,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            lines: vec![None; rows as usize],
        }
    }

    pub fn line(mut self, row: u16, content: &str) -> Self {
        let row = row as usize;
        assert!(row < self.rows as usize, "row {row} is out of bounds");

        self.lines[row] = Some(pad_to_cols(self.cols, content));
        self
    }

    pub fn build(self) -> Vec<String> {
        let blank = " ".repeat(self.cols as usize);
        self.lines
            .into_iter()
            .map(|line| line.unwrap_or_else(|| blank.clone()))
            .collect()
    }
}

pub fn pad_to_cols(cols: u16, content: &str) -> String {
    let width = content.width();
    assert!(
        width <= cols as usize,
        "line width {width} exceeds terminal width {cols}"
    );

    let mut line = String::from(content);
    line.push_str(&" ".repeat(cols as usize - width));
    line
}

pub fn assert_screen_eq(expected: &[String], actual: &[String]) {
    if actual == expected {
        return;
    }

    panic!("{}", format_screen_diff(expected, actual));
}

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

fn format_screen_line(line: Option<&String>) -> String {
    match line {
        Some(line) => format!("|{}|", line.replace(' ', "·")),
        None => "<missing>".to_string(),
    }
}

fn format_screen(lines: &[String], total_rows: usize) -> Vec<String> {
    (0..total_rows)
        .map(|row| format!("  r{row:02} {}", format_screen_line(lines.get(row))))
        .collect()
}
