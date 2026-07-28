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
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use crossbeam_skiplist::SkipMap;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    terminal::Terminal,
    widget::{CreatedGraphemes, ScreenPosition, WidgetPosition},
};

mod layout;
use layout::LayoutSnapshot;
pub use layout::{PreparedLayout, RendererLayout};

const RESIZE_SIZE_STABILITY: Duration = Duration::from_millis(20);
const RESIZE_SIZE_POLL: Duration = Duration::from_millis(5);

/// SharedRenderer is a type alias for an Arc-wrapped Renderer, allowing for shared ownership and concurrency.
pub type SharedRenderer<K> = Arc<Renderer<K>>;

/// Renderer stores widget content, lays it out, and draws it to a terminal.
pub struct Renderer<K: Clone + Ord + Send + Sync + 'static> {
    terminal: AsyncMutex<Terminal>,
    contents: SkipMap<K, CreatedGraphemes>,
    layout_engine: Mutex<RendererLayout<K>>,
    layout: RwLock<Option<LayoutSnapshot<K>>>,
    last_terminal_size: Mutex<Option<(u16, u16)>>,
}

impl<K: Clone + Ord + Send + Sync + 'static> Renderer<K> {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            terminal: AsyncMutex::new(Terminal::new(crate::crossterm::cursor::position()?)),
            contents: SkipMap::new(),
            layout_engine: Mutex::new(RendererLayout::default()),
            layout: RwLock::new(None),
            last_terminal_size: Mutex::new(None),
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
        let mut layout_engine = self.layout_engine.lock().expect("layout lock poisoned");
        items.into_iter().for_each(|index| {
            self.contents.remove(&index);
            layout_engine.remove(&index);
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
        let mut size = crate::crossterm::terminal::size()?;
        let previous_size = *self
            .last_terminal_size
            .lock()
            .expect("terminal size lock poisoned");
        if previous_size.is_some_and(|previous| previous != size) {
            size = wait_for_terminal_size_stability(size).await?;
        }

        loop {
            let (terminal_width, terminal_height) = size;
            let prepared = self
                .layout_engine
                .lock()
                .expect("layout lock poisoned")
                .layout(contents.iter().cloned(), terminal_width, terminal_height)?;

            let panes = prepared.panes();
            terminal.draw_rows_at_height(&panes, terminal_height)?;
            drop(panes);

            let current_size = crate::crossterm::terminal::size()?;
            if current_size != size {
                size = wait_for_terminal_size_stability(current_size).await?;
                continue;
            }

            let origin = ScreenPosition {
                row: terminal.position.1,
                column: terminal.position.0,
            };
            *self.layout.write().expect("layout lock poisoned") =
                Some(prepared.into_snapshot(origin));
            *self
                .last_terminal_size
                .lock()
                .expect("terminal size lock poisoned") = Some(size);

            return Ok(());
        }
    }
}

async fn wait_for_terminal_size_stability(mut previous: (u16, u16)) -> std::io::Result<(u16, u16)> {
    let mut stable_since = Instant::now();

    loop {
        tokio::time::sleep(RESIZE_SIZE_POLL).await;
        let current = crate::crossterm::terminal::size()?;
        if current != previous {
            previous = current;
            stable_since = Instant::now();
        } else if stable_since.elapsed() >= RESIZE_SIZE_STABILITY {
            return Ok(current);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grapheme::StyledGraphemes, widget::WidgetViewport};

    mod hit_test {
        use super::*;

        #[test]
        fn maps_screen_positions_back_to_widget_positions() {
            let renderer = Renderer {
                terminal: AsyncMutex::new(Terminal::new((0, 0))),
                contents: SkipMap::new(),
                layout_engine: Mutex::new(RendererLayout::default()),
                layout: RwLock::new(Some(LayoutSnapshot {
                    origin: ScreenPosition { row: 3, column: 0 },
                    terminal_width: 20,
                    entries: vec![layout::LayoutEntry {
                        index: 7usize,
                        viewport: WidgetViewport {
                            screen_row: 3,
                            height: 2,
                            content_row: 1,
                        },
                        rows: vec![
                            layout::VisualRow {
                                content_row: 0,
                                content_column: 0,
                                graphemes: StyledGraphemes::from("hidden"),
                            },
                            layout::VisualRow {
                                content_row: 1,
                                content_column: 0,
                                graphemes: StyledGraphemes::from("first"),
                            },
                            layout::VisualRow {
                                content_row: 2,
                                content_column: 0,
                                graphemes: StyledGraphemes::from("second"),
                            },
                        ],
                    }],
                })),
                last_terminal_size: Mutex::new(None),
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
}
