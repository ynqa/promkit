#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub mod yaml;

#[cfg(feature = "tree")]
#[cfg_attr(docsrs, doc(cfg(feature = "tree")))]
pub mod tree;

use std::cell::Cell;

use promkit_core::grapheme::StyledGraphemes;

fn with_line_numbers(
    lines: Vec<StyledGraphemes>,
    line_numbers: Vec<usize>,
    line_count: usize,
) -> Vec<StyledGraphemes> {
    debug_assert_eq!(lines.len(), line_numbers.len());
    let width = line_count.max(1).to_string().len();

    line_numbers
        .into_iter()
        .zip(lines)
        .map(|(line_number, line)| {
            [
                StyledGraphemes::from(format!("{line_number:>width$} ")),
                line,
            ]
            .into_iter()
            .collect()
        })
        .collect()
}

/// Container type of structured widget.
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerType {
    Object,
    Array,
}

impl ContainerType {
    /// Opening string of the container.
    pub fn open_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{",
            ContainerType::Array => "[",
        }
    }

    /// Closing string of the container.
    pub fn close_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "}",
            ContainerType::Array => "]",
        }
    }

    /// Empty string of the container.
    pub fn empty_str(&self) -> &'static str {
        match self {
            ContainerType::Object => "{}",
            ContainerType::Array => "[]",
        }
    }

    /// Collapsed preview string of the container.
    pub fn collapsed_preview(&self) -> &'static str {
        match self {
            ContainerType::Object => "{…}",
            ContainerType::Array => "[…]",
        }
    }
}

/// Container node of structured widget, which can be an empty container,
/// an opening container, or a closing container.
#[derive(Clone, Debug, PartialEq)]
pub enum ContainerNode {
    /// An empty container (e.g., `{}` or `[]`).
    Empty { typ: ContainerType },
    /// An opening container (e.g., `{` or `[`), with information
    /// about whether it's collapsed and its corresponding closing index.
    Open {
        typ: ContainerType,
        collapsed: bool,
        close_index: usize,
    },
    /// A closing container (e.g., `}` or `]`), with information
    /// about whether it's collapsed and its corresponding opening index.
    Close {
        typ: ContainerType,
        collapsed: bool,
        open_index: usize,
    },
}

pub trait PrettyRender {
    /// Render the row as a pretty-printed string with the specified indentation level.
    fn render_pretty(&self, indent: usize) -> String;
}

/// Common row navigation and visibility operations for structured views.
pub trait RowOperation {
    /// Row element type handled by the operation target.
    type Row;

    /// Move the cursor one row upward from `current`.
    fn up(&self, current: usize) -> usize;

    /// Move the cursor to the first visible row.
    fn head(&self) -> usize;

    /// Move the cursor one row downward from `current`.
    fn down(&self, current: usize) -> usize;

    /// Move the cursor to the last visible row.
    fn tail(&self) -> usize;

    /// Toggle collapse/expand state at `current` and return the next cursor position.
    fn toggle(&mut self, current: usize) -> usize;

    /// Set collapsed visibility for all container rows.
    fn set_rows_visibility(&mut self, collapsed: bool);

    /// Extract up to `n` visible rows starting from `current`.
    fn extract(&self, current: usize, n: usize) -> Vec<Self::Row>;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProjectionViewport {
    start: usize,
    cursor: usize,
    cursor_row: usize,
    height: usize,
    initialized: bool,
}

pub(crate) fn projection_viewport<R>(
    rows: &Vec<R>,
    cursor: usize,
    height: usize,
    viewport: &Cell<ProjectionViewport>,
) -> (usize, usize)
where
    Vec<R>: RowOperation<Row = R>,
{
    if rows.is_empty() || height == 0 {
        return (rows.head(), 0);
    }

    let previous = viewport.get();
    let (start, cursor_row) = if !previous.initialized {
        place_cursor_from(rows, rows.head(), cursor, height)
    } else if previous.height != height {
        place_cursor_from(rows, previous.start, cursor, height)
    } else if cursor == previous.cursor {
        (previous.start, previous.cursor_row)
    } else if cursor > previous.cursor && is_next(rows, previous.cursor, cursor) {
        if previous.cursor_row + 1 < height {
            (previous.start, previous.cursor_row + 1)
        } else {
            (rows.down(previous.start), height - 1)
        }
    } else if cursor < previous.cursor && is_previous(rows, previous.cursor, cursor) {
        if previous.cursor_row > 0 {
            (previous.start, previous.cursor_row - 1)
        } else {
            (cursor, 0)
        }
    } else {
        place_cursor_from(rows, previous.start, cursor, height)
    };

    viewport.set(ProjectionViewport {
        start,
        cursor,
        cursor_row,
        height,
        initialized: true,
    });
    (start, cursor_row)
}

#[inline]
fn is_next<R>(rows: &Vec<R>, previous: usize, cursor: usize) -> bool
where
    Vec<R>: RowOperation<Row = R>,
{
    cursor == previous.saturating_add(1) || rows.down(previous) == cursor
}

#[inline]
fn is_previous<R>(rows: &Vec<R>, previous: usize, cursor: usize) -> bool
where
    Vec<R>: RowOperation<Row = R>,
{
    cursor.saturating_add(1) == previous || rows.up(previous) == cursor
}

fn place_cursor_from<R>(rows: &Vec<R>, start: usize, cursor: usize, height: usize) -> (usize, usize)
where
    Vec<R>: RowOperation<Row = R>,
{
    if cursor < start {
        return (cursor, 0);
    }

    let mut position = start;
    for cursor_row in 0..height {
        if position == cursor {
            return (start, cursor_row);
        }
        let next = rows.down(position);
        if next == position {
            break;
        }
        position = next;
    }

    let mut start = cursor;
    let mut cursor_row = 0;
    while cursor_row + 1 < height {
        let previous = rows.up(start);
        if previous == start {
            break;
        }
        start = previous;
        cursor_row += 1;
    }
    (start, cursor_row)
}
