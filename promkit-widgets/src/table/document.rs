use std::{cell::Cell, io::Read, ops::Range};

use unicode_width::UnicodeWidthChar;

/// CSV parsing options used when constructing a table document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CsvOptions {
    delimiter: u8,
    has_headers: bool,
}

impl CsvOptions {
    /// Sets the one-byte field delimiter.
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Controls whether the first record is treated as a header.
    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellSpan {
    start: u32,
    len: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Projection {
    pub(crate) first_row: usize,
    pub(crate) horizontal_offset: usize,
    pub(crate) max_horizontal_offset: usize,
    pub(crate) body_height: usize,
    pub(crate) width: usize,
    pub(crate) header_visible: bool,
}

/// Compact, navigable tabular data.
///
/// Cell text is held in one contiguous arena. Rows are rectangular, so a cell
/// requires only a flat span index instead of nested vectors and per-cell
/// string allocations.
#[derive(Clone)]
pub struct Document {
    arena: String,
    cells: Box<[CellSpan]>,
    column_widths: Box<[usize]>,
    cell_content_width: usize,
    record_count: usize,
    column_count: usize,
    has_header: bool,
    position: usize,
    horizontal_offset: Cell<usize>,
    projection: Cell<Projection>,
}

impl Document {
    /// Parses a CSV stream into the compact table representation.
    pub fn from_csv<R: Read>(reader: R, options: CsvOptions) -> Result<Self, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(options.delimiter)
            .has_headers(false)
            .flexible(false)
            .from_reader(reader);
        let mut record = csv::StringRecord::new();
        let mut arena = String::new();
        let mut cells = Vec::new();
        let mut column_widths = Vec::new();
        let mut record_count = 0usize;
        let mut column_count = 0usize;

        while reader.read_record(&mut record)? {
            if record_count == 0 {
                column_count = record.len();
                column_widths.resize(column_count, 0);
            }

            for (column, field) in record.iter().enumerate() {
                let start = u32::try_from(arena.len()).map_err(|_| {
                    csv::Error::from(std::io::Error::other("table cell arena exceeds 4 GiB"))
                })?;
                let len = u32::try_from(field.len()).map_err(|_| {
                    csv::Error::from(std::io::Error::other("table cell exceeds 4 GiB"))
                })?;

                arena.push_str(field);
                cells.push(CellSpan { start, len });
                column_widths[column] = column_widths[column].max(display_width(field));
            }
            record_count += 1;
        }

        let has_header = options.has_headers && record_count > 0;
        let cell_content_width = column_widths
            .iter()
            .copied()
            .map(|width| width.max(1))
            .fold(0usize, usize::saturating_add);
        Ok(Self {
            arena,
            cells: cells.into_boxed_slice(),
            column_widths: column_widths.into_boxed_slice(),
            cell_content_width,
            record_count,
            column_count,
            has_header,
            position: 0,
            horizontal_offset: Cell::new(0),
            projection: Cell::default(),
        })
    }

    /// Returns the number of body rows, excluding the header.
    pub fn row_count(&self) -> usize {
        self.record_count - usize::from(self.has_header)
    }

    /// Returns the number of columns.
    pub fn column_count(&self) -> usize {
        self.column_count
    }

    /// Returns whether the document contains a header record.
    pub fn has_header(&self) -> bool {
        self.has_header
    }

    /// Returns the selected body row.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Returns the horizontal viewport offset in terminal display cells.
    pub fn horizontal_offset(&self) -> usize {
        self.horizontal_offset.get()
    }

    /// Returns a header cell.
    pub fn header_cell(&self, column: usize) -> Option<&str> {
        self.has_header
            .then(|| self.record_cell(0, column))
            .flatten()
    }

    /// Returns a body cell.
    pub fn cell(&self, row: usize, column: usize) -> Option<&str> {
        let record = row.checked_add(usize::from(self.has_header))?;
        self.record_cell(record, column)
    }

    pub(crate) fn column_width(&self, column: usize) -> Option<usize> {
        self.column_widths
            .get(column)
            .copied()
            .map(|width| width.max(1))
    }

    pub(crate) fn content_width(&self, separator_width: usize) -> usize {
        self.cell_content_width
            .saturating_add(separator_width.saturating_mul(self.column_count.saturating_sub(1)))
    }

    pub(crate) fn set_horizontal_offset(&self, horizontal_offset: usize) {
        self.horizontal_offset.set(horizontal_offset);
    }

    pub(crate) fn projected_rows(&self, body_height: usize) -> Range<usize> {
        let row_count = self.row_count();
        if row_count == 0 || body_height == 0 {
            return 0..0;
        }

        let previous = self.projection.get();
        let mut first = previous.first_row.min(row_count - 1);
        if self.position < first {
            first = self.position;
        } else if self.position >= first.saturating_add(body_height) {
            first = self.position.saturating_add(1).saturating_sub(body_height);
        }

        first..first.saturating_add(body_height).min(row_count)
    }

    pub(crate) fn set_projection(&self, projection: Projection) {
        self.projection.set(projection);
    }

    pub(crate) fn projection(&self) -> Projection {
        self.projection.get()
    }

    /// Moves the selected row up.
    pub fn up(&mut self) -> bool {
        let previous = self.position;
        self.position = self.position.saturating_sub(1);
        self.position != previous
    }

    /// Moves the selected row down.
    pub fn down(&mut self) -> bool {
        let previous = self.position;
        if self.position + 1 < self.row_count() {
            self.position += 1;
        }
        self.position != previous
    }

    /// Moves to the first body row.
    pub fn head(&mut self) -> bool {
        let previous = self.position;
        self.position = 0;
        self.position != previous
    }

    /// Moves to the last body row.
    pub fn tail(&mut self) -> bool {
        let previous = self.position;
        self.position = self.row_count().saturating_sub(1);
        self.position != previous
    }

    /// Scrolls the table one terminal display cell left.
    pub fn scroll_left(&mut self) -> bool {
        let previous = self.horizontal_offset.get();
        self.horizontal_offset.set(previous.saturating_sub(1));
        self.horizontal_offset.get() != previous
    }

    /// Scrolls the table one terminal display cell right.
    ///
    /// The right boundary is established by the latest viewport projection.
    pub fn scroll_right(&mut self) -> bool {
        let previous = self.horizontal_offset.get();
        let maximum = self.projection.get().max_horizontal_offset;
        if previous < maximum {
            self.horizontal_offset.set(previous + 1);
        }
        self.horizontal_offset.get() != previous
    }

    /// Scrolls horizontally to the beginning of the table.
    pub fn scroll_to_start(&mut self) -> bool {
        let previous = self.horizontal_offset.get();
        self.horizontal_offset.set(0);
        previous != 0
    }

    /// Scrolls horizontally to the end of the latest viewport projection.
    pub fn scroll_to_end(&mut self) -> bool {
        let previous = self.horizontal_offset.get();
        self.horizontal_offset
            .set(self.projection.get().max_horizontal_offset);
        self.horizontal_offset.get() != previous
    }

    fn record_cell(&self, record: usize, column: usize) -> Option<&str> {
        if record >= self.record_count || column >= self.column_count {
            return None;
        }
        let span = self.cells.get(record * self.column_count + column)?;
        let start = span.start as usize;
        self.arena.get(start..start + span.len as usize)
    }
}

pub(crate) fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| display_char(ch).width().unwrap_or(0))
        .sum()
}

pub(crate) fn display_char(ch: char) -> char {
    match ch {
        '\n' => '↵',
        '\r' => '␍',
        '\t' => '⇥',
        ch if ch.is_control() => '�',
        ch => ch,
    }
}
