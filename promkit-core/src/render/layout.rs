use std::collections::BTreeMap;

use crate::{
    grapheme::StyledGraphemes,
    widget::{
        ContentPosition, CreatedGraphemes, ScreenPosition, VisualPosition, WidgetViewport,
        WidthMode,
    },
};

/// Terminal-size-dependent renderer layout without terminal I/O.
///
/// The layout keeps each pane's vertical viewport offset between calls. This
/// mirrors [`super::Renderer`] behavior while allowing layout performance to be
/// measured independently from terminal size queries and stdout writes.
#[derive(Debug)]
pub struct RendererLayout<K> {
    viewport_rows: BTreeMap<K, usize>,
}

impl<K> Default for RendererLayout<K> {
    fn default() -> Self {
        Self {
            viewport_rows: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct VisualRow {
    pub(super) content_row: usize,
    pub(super) content_column: usize,
    pub(super) graphemes: StyledGraphemes,
}

#[derive(Clone, Debug)]
pub(super) struct LayoutEntry<K> {
    pub(super) index: K,
    pub(super) viewport: WidgetViewport,
    pub(super) rows: Vec<VisualRow>,
}

#[derive(Clone, Debug)]
pub(super) struct LayoutSnapshot<K> {
    pub(super) origin: ScreenPosition,
    pub(super) terminal_width: u16,
    pub(super) entries: Vec<LayoutEntry<K>>,
}

/// A renderer frame after wrapping, viewport allocation, and clipping.
///
/// It intentionally exposes only aggregate information and the visible panes.
/// Coordinate mappings are installed by [`super::Renderer`] after the panes
/// have been drawn at a known screen origin.
#[derive(Debug)]
pub struct PreparedLayout<K> {
    pub(super) terminal_width: u16,
    pub(super) entries: Vec<LayoutEntry<K>>,
    pub(super) panes: Vec<Vec<StyledGraphemes>>,
}

impl<K> PreparedLayout<K> {
    /// Returns the visible rows grouped by renderer pane.
    pub fn panes(&self) -> &[Vec<StyledGraphemes>] {
        &self.panes
    }

    /// Returns the number of non-empty panes allocated in this frame.
    pub fn pane_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the number of visual rows produced before viewport clipping.
    pub fn visual_row_count(&self) -> usize {
        self.entries.iter().map(|entry| entry.rows.len()).sum()
    }

    /// Returns the number of rows that will be written to the terminal.
    pub fn visible_row_count(&self) -> usize {
        self.panes.iter().map(Vec::len).sum()
    }

    pub(super) fn into_snapshot(mut self, origin: ScreenPosition) -> LayoutSnapshot<K> {
        let mut screen_row = origin.row;
        for entry in &mut self.entries {
            entry.viewport.screen_row = screen_row;
            screen_row = screen_row.saturating_add(entry.viewport.height);
        }

        LayoutSnapshot {
            origin,
            terminal_width: self.terminal_width,
            entries: self.entries,
        }
    }
}

impl<K: Clone + Ord> RendererLayout<K> {
    /// Lays out a renderer frame for a known terminal size.
    ///
    /// This performs the same content wrapping, truncation, pane allocation,
    /// cursor scrolling, and visible-row cloning used by
    /// [`super::Renderer::render`], but performs no terminal I/O.
    pub fn layout<I>(
        &mut self,
        contents: I,
        terminal_width: u16,
        terminal_height: u16,
    ) -> anyhow::Result<PreparedLayout<K>>
    where
        I: IntoIterator<Item = (K, CreatedGraphemes)>,
    {
        let laid_out = contents
            .into_iter()
            .map(|(index, created)| {
                let rows = layout_content(&created, terminal_width as usize);
                (index, created, rows)
            })
            .filter(|(_, created, rows)| !rows.is_empty() && created.layout.max_height != Some(0))
            .collect::<Vec<_>>();

        if laid_out.len() > terminal_height as usize {
            return Err(anyhow::anyhow!("Insufficient space to display all panes"));
        }

        let mut entries = Vec::with_capacity(laid_out.len());
        let mut panes = Vec::with_capacity(laid_out.len());
        let mut used_height = 0usize;

        for (pane_index, (index, created, rows)) in laid_out.iter().enumerate() {
            let panes_after = laid_out.len().saturating_sub(pane_index + 1);
            let available = (terminal_height as usize)
                .saturating_sub(used_height)
                .saturating_sub(panes_after);
            let desired = created
                .layout
                .max_height
                .unwrap_or(rows.len())
                .min(rows.len());
            let height = desired.min(available).max(1);
            used_height = used_height.saturating_add(height);

            let mut viewport = WidgetViewport {
                height: height as u16,
                content_row: self.viewport_rows.get(index).copied().unwrap_or_default(),
                ..Default::default()
            };

            let max_content_row = rows.len().saturating_sub(height);
            viewport.content_row = viewport.content_row.min(max_content_row);

            if let Some(cursor) = created.cursor
                && let Some(position) = visual_position(rows, cursor)
            {
                viewport.scroll_to_include(position);
                viewport.content_row = viewport.content_row.min(max_content_row);
            }

            self.viewport_rows
                .insert(index.clone(), viewport.content_row);
            let visible_rows = rows
                .iter()
                .skip(viewport.content_row)
                .take(height)
                .map(|row| row.graphemes.clone())
                .collect::<Vec<_>>();

            panes.push(visible_rows);
            entries.push(LayoutEntry {
                index: index.clone(),
                viewport,
                rows: rows.clone(),
            });
        }

        Ok(PreparedLayout {
            terminal_width,
            entries,
            panes,
        })
    }

    pub(super) fn remove(&mut self, index: &K) {
        self.viewport_rows.remove(index);
    }
}

fn layout_content(created: &CreatedGraphemes, width: usize) -> Vec<VisualRow> {
    if width == 0 {
        return Vec::new();
    }

    created
        .graphemes
        .logical_lines()
        .into_iter()
        .enumerate()
        .flat_map(|(content_row, line)| match created.layout.width_mode {
            WidthMode::Wrap => wrap_line(content_row, line, width),
            WidthMode::Truncate => vec![VisualRow {
                content_row,
                content_column: 0,
                graphemes: line.truncated_line_with_ellipsis(width, &StyledGraphemes::from("…")),
            }],
        })
        .collect()
}

fn wrap_line(content_row: usize, line: StyledGraphemes, width: usize) -> Vec<VisualRow> {
    if line.is_empty() {
        return vec![VisualRow {
            content_row,
            content_column: 0,
            graphemes: line,
        }];
    }

    let mut rows = Vec::new();
    let mut row = StyledGraphemes::default();
    let mut row_width = 0usize;
    let mut content_column = 0usize;
    let mut row_column = 0usize;

    for grapheme in line.iter() {
        let grapheme_width = grapheme.width();
        if grapheme_width > width {
            if !row.is_empty() {
                rows.push(VisualRow {
                    content_row,
                    content_column: row_column,
                    graphemes: row,
                });
                row = StyledGraphemes::default();
                row_width = 0;
            }

            // Keep the replacement on its own visual row: it occupies one screen
            // cell while the original grapheme still advances by its logical width.
            rows.push(VisualRow {
                content_row,
                content_column,
                graphemes: StyledGraphemes::from("…"),
            });
            content_column = content_column.saturating_add(grapheme_width);
            row_column = content_column;
            continue;
        }

        if !row.is_empty() && row_width.saturating_add(grapheme_width) > width {
            rows.push(VisualRow {
                content_row,
                content_column: row_column,
                graphemes: row,
            });
            row = StyledGraphemes::default();
            row_width = 0;
            row_column = content_column;
        }

        row.push_back(grapheme.clone());
        row_width = row_width.saturating_add(grapheme_width);
        content_column = content_column.saturating_add(grapheme_width);
    }

    if !row.is_empty() {
        rows.push(VisualRow {
            content_row,
            content_column: row_column,
            graphemes: row,
        });
    }

    rows
}

pub(super) fn visual_position(
    rows: &[VisualRow],
    position: ContentPosition,
) -> Option<VisualPosition> {
    let matching = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.content_row == position.row)
        .collect::<Vec<_>>();

    let (row_index, row) = matching
        .iter()
        .copied()
        .find(|(_, row)| {
            let end = row.content_column.saturating_add(row.graphemes.widths());
            position.column >= row.content_column && position.column < end
        })
        .or_else(|| matching.last().copied())?;

    Some(VisualPosition {
        row: row_index,
        column: position.column.saturating_sub(row.content_column),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::WidgetLayout;

    #[test]
    fn layout_maps_logical_cursor_to_wrapped_row() {
        let created = CreatedGraphemes {
            graphemes: StyledGraphemes::from("abcdefghij"),
            cursor: Some(ContentPosition { row: 0, column: 8 }),
            ..Default::default()
        };
        let rows = layout_content(&created, 4);

        assert_eq!(rows.len(), 3);
        assert_eq!(
            visual_position(&rows, created.cursor.unwrap()),
            Some(VisualPosition { row: 2, column: 0 })
        );
    }

    #[test]
    fn wrap_preserves_a_grapheme_wider_than_the_terminal() {
        let created = CreatedGraphemes {
            graphemes: StyledGraphemes::from("界"),
            cursor: Some(ContentPosition { row: 0, column: 0 }),
            ..Default::default()
        };

        let rows = layout_content(&created, 1);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].content_row, 0);
        assert_eq!(rows[0].content_column, 0);
        assert_eq!(rows[0].graphemes.to_string(), "…");
        assert_eq!(
            visual_position(&rows, created.cursor.unwrap()),
            Some(VisualPosition { row: 0, column: 0 })
        );
    }

    #[test]
    fn wrap_preserves_columns_after_a_grapheme_wider_than_the_terminal() {
        let created = CreatedGraphemes {
            graphemes: StyledGraphemes::from("界a"),
            cursor: Some(ContentPosition { row: 0, column: 2 }),
            ..Default::default()
        };

        let rows = layout_content(&created, 1);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].content_column, 0);
        assert_eq!(rows[0].graphemes.to_string(), "…");
        assert_eq!(rows[1].content_column, 2);
        assert_eq!(rows[1].graphemes.to_string(), "a");
        assert_eq!(
            visual_position(&rows, created.cursor.unwrap()),
            Some(VisualPosition { row: 1, column: 0 })
        );
    }

    #[test]
    fn truncate_keeps_one_visual_row_per_logical_row() {
        let created = CreatedGraphemes {
            graphemes: StyledGraphemes::from("abcdefghij\nsecond"),
            layout: WidgetLayout {
                width_mode: WidthMode::Truncate,
                ..Default::default()
            },
            cursor: None,
        };
        let rows = layout_content(&created, 4);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].graphemes.to_string(), "abc…");
        assert_eq!(rows[1].graphemes.to_string(), "sec…");
    }

    #[test]
    fn allocates_height_and_preserves_viewport_between_frames() {
        let created = CreatedGraphemes {
            graphemes: StyledGraphemes::from("first\nsecond\nthird"),
            layout: WidgetLayout {
                max_height: Some(2),
                ..Default::default()
            },
            cursor: Some(ContentPosition { row: 2, column: 0 }),
        };
        let mut layout = RendererLayout::default();

        let first = layout.layout([(0, created.clone())], 80, 24).unwrap();
        assert_eq!(first.pane_count(), 1);
        assert_eq!(first.visual_row_count(), 3);
        assert_eq!(first.visible_row_count(), 2);
        assert_eq!(first.panes()[0][0].to_string(), "second");

        let second = layout.layout([(0, created)], 80, 24).unwrap();
        assert_eq!(second.panes()[0][0].to_string(), "second");
    }
}
