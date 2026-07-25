//! Terminal-dependent layout, viewport management, drawing, and hit testing.
//!
//! [`Renderer`] stores widget outputs in key order. During [`Renderer::render`]
//! it reads the current terminal size, wraps or truncates every logical content
//! row, allocates vertical viewports, scrolls each keyed viewport just enough to
//! include its cursor, and delegates the resulting visible rows to the terminal.
//!
//! Empty items and items with `max_height == Some(0)` do not occupy space.
//! Remaining items are allocated in key order while reserving at least one row
//! for every later non-empty item. A terminal that cannot provide one row per
//! non-empty item produces an error.
//!
//! A successful render saves a layout snapshot. [`Renderer::hit_test`] and
//! [`Renderer::screen_position`] always use that snapshot, so event handling maps
//! positions against what was actually drawn rather than against newer,
//! not-yet-rendered content.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};

use crossbeam_skiplist::SkipMap;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    grapheme::StyledGraphemes,
    terminal::Terminal,
    widget::{
        ContentPosition, CreatedGraphemes, ScreenPosition, VisualPosition, WidgetPosition,
        WidgetViewport, WidthMode,
    },
};

/// SharedRenderer is a type alias for an Arc-wrapped Renderer, allowing for shared ownership and concurrency.
pub type SharedRenderer<K> = Arc<Renderer<K>>;

#[derive(Clone, Debug)]
struct VisualRow {
    content_row: usize,
    content_column: usize,
    graphemes: StyledGraphemes,
}

#[derive(Clone, Debug)]
struct LayoutEntry<K> {
    index: K,
    viewport: WidgetViewport,
    rows: Vec<VisualRow>,
}

#[derive(Clone, Debug)]
struct LayoutSnapshot<K> {
    origin: ScreenPosition,
    terminal_width: u16,
    entries: Vec<LayoutEntry<K>>,
}

/// Renderer stores widget content, lays it out, and draws it to a terminal.
pub struct Renderer<K: Clone + Ord + Send + Sync + 'static> {
    terminal: AsyncMutex<Terminal>,
    contents: SkipMap<K, CreatedGraphemes>,
    viewport_rows: Mutex<BTreeMap<K, usize>>,
    layout: RwLock<Option<LayoutSnapshot<K>>>,
}

impl<K: Clone + Ord + Send + Sync + 'static> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: AsyncMutex::new(Terminal {
                position: crate::crossterm::cursor::position()?,
            }),
            contents: SkipMap::new(),
            viewport_rows: Mutex::new(BTreeMap::new()),
            layout: RwLock::new(None),
        })
    }

    pub async fn try_new_with_graphemes<I, G>(init: I, draw: bool) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = (K, G)>,
        G: Into<CreatedGraphemes>,
    {
        let renderer = Self::try_new()?;
        renderer.update(init);
        if draw {
            renderer.render().await?;
        }
        Ok(renderer)
    }

    pub fn update<I, G>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = (K, G)>,
        G: Into<CreatedGraphemes>,
    {
        items.into_iter().for_each(|(index, graphemes)| {
            self.contents.insert(index, graphemes.into());
        });
        self
    }

    pub fn remove<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = K>,
    {
        let mut viewport_rows = self.viewport_rows.lock().expect("viewport lock poisoned");
        items.into_iter().for_each(|index| {
            self.contents.remove(&index);
            viewport_rows.remove(&index);
        });
        self
    }

    /// Returns the content position under a terminal screen position.
    ///
    /// This always uses the layout from the most recently completed render.
    pub fn hit_test(&self, position: ScreenPosition) -> Option<WidgetPosition<K>> {
        let layout = self.layout.read().ok()?;
        let layout = layout.as_ref()?;

        if position.column >= layout.terminal_width {
            return None;
        }

        let entry = layout.entries.iter().find(|entry| {
            let start = entry.viewport.screen_row;
            let end = start.saturating_add(entry.viewport.height);
            position.row >= start && position.row < end
        })?;

        let relative_row = position.row.saturating_sub(entry.viewport.screen_row) as usize;
        let visual_row = entry.viewport.content_row.saturating_add(relative_row);
        let row = entry.rows.get(visual_row)?;

        let screen_column = if position.row == layout.origin.row {
            position.column.checked_sub(layout.origin.column)?
        } else {
            position.column
        };

        Some(WidgetPosition {
            index: entry.index.clone(),
            row: row.content_row,
            column: row.content_column.saturating_add(screen_column as usize),
        })
    }

    /// Returns the screen position for a widget content position when it is visible.
    pub fn screen_position(&self, position: WidgetPosition<K>) -> Option<ScreenPosition> {
        let layout = self.layout.read().ok()?;
        let layout = layout.as_ref()?;
        let entry = layout
            .entries
            .iter()
            .find(|entry| entry.index == position.index)?;

        let matching_rows = entry
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.content_row == position.row)
            .collect::<Vec<_>>();

        let (visual_row, row) = matching_rows
            .iter()
            .copied()
            .find(|(_, row)| {
                let end = row.content_column.saturating_add(row.graphemes.widths());
                position.column >= row.content_column && position.column < end
            })
            .or_else(|| matching_rows.last().copied())?;

        let viewport_end = entry
            .viewport
            .content_row
            .saturating_add(entry.viewport.height as usize);
        if visual_row < entry.viewport.content_row || visual_row >= viewport_end {
            return None;
        }

        let row_offset = visual_row.saturating_sub(entry.viewport.content_row) as u16;
        let screen_row = entry.viewport.screen_row.saturating_add(row_offset);
        let row_origin_column = if screen_row == layout.origin.row {
            layout.origin.column
        } else {
            0
        };
        let column_offset = position.column.saturating_sub(row.content_column);
        let column_offset = u16::try_from(column_offset).ok()?;
        let column = row_origin_column.saturating_add(column_offset);

        (column < layout.terminal_width).then_some(ScreenPosition {
            row: screen_row,
            column,
        })
    }

    /// Lays out all current widget outputs and renders their visible viewports.
    ///
    /// Viewport offsets persist by item key. They remain stable while a cursor is
    /// visible, move only when it crosses a viewport edge, and are clamped when
    /// content or terminal dimensions shrink.
    pub async fn render(&self) -> anyhow::Result<()> {
        let contents = self
            .contents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect::<Vec<_>>();

        let mut terminal = self.terminal.lock().await;
        let (terminal_width, terminal_height) = crate::crossterm::terminal::size()?;
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

        let mut viewport_rows = self.viewport_rows.lock().expect("viewport lock poisoned");
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
                content_row: viewport_rows.get(index).copied().unwrap_or_default(),
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

            viewport_rows.insert(index.clone(), viewport.content_row);
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
        drop(viewport_rows);

        terminal.draw_rows(&panes)?;

        let origin = ScreenPosition {
            row: terminal.position.1,
            column: terminal.position.0,
        };
        let mut screen_row = origin.row;
        for entry in &mut entries {
            entry.viewport.screen_row = screen_row;
            screen_row = screen_row.saturating_add(entry.viewport.height);
        }

        *self.layout.write().expect("layout lock poisoned") = Some(LayoutSnapshot {
            origin,
            terminal_width,
            entries,
        });

        Ok(())
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

fn visual_position(rows: &[VisualRow], position: ContentPosition) -> Option<VisualPosition> {
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
    use crate::widget::{WidgetLayout, WidthMode};

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
    fn hit_test_and_screen_position_round_trip() {
        let renderer = Renderer {
            terminal: AsyncMutex::new(Terminal { position: (0, 0) }),
            contents: SkipMap::new(),
            viewport_rows: Mutex::new(BTreeMap::new()),
            layout: RwLock::new(Some(LayoutSnapshot {
                origin: ScreenPosition { row: 3, column: 0 },
                terminal_width: 20,
                entries: vec![LayoutEntry {
                    index: 7usize,
                    viewport: WidgetViewport {
                        screen_row: 3,
                        height: 2,
                        content_row: 1,
                    },
                    rows: vec![
                        VisualRow {
                            content_row: 0,
                            content_column: 0,
                            graphemes: StyledGraphemes::from("hidden"),
                        },
                        VisualRow {
                            content_row: 1,
                            content_column: 0,
                            graphemes: StyledGraphemes::from("first"),
                        },
                        VisualRow {
                            content_row: 2,
                            content_column: 0,
                            graphemes: StyledGraphemes::from("second"),
                        },
                    ],
                }],
            })),
        };

        let screen = ScreenPosition { row: 4, column: 2 };
        let widget = renderer.hit_test(screen).unwrap();
        assert_eq!(
            widget,
            WidgetPosition {
                index: 7,
                row: 2,
                column: 2,
            }
        );
        assert_eq!(renderer.screen_position(widget), Some(screen));
    }
}
