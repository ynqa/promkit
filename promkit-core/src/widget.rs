//! Width-independent widget output and the coordinate types used during layout.
//!
//! Widgets project state into their complete styled content without receiving a
//! terminal size or viewport. Alongside that content they return a [`WidgetLayout`]
//! hint and, when applicable, a logical [`ContentPosition`] for the cursor.
//! [`crate::render::Renderer`] owns terminal-dependent wrapping, truncation,
//! vertical viewport allocation, and scrolling.
//!
//! Positions deliberately have three coordinate spaces:
//!
//! - [`ContentPosition`] addresses newline-delimited widget content. Its column is
//!   measured in terminal display cells.
//! - [`VisualPosition`] addresses the rows produced after terminal-width layout.
//! - [`ScreenPosition`] addresses absolute terminal cells.
//!
//! The renderer records the mapping between these spaces after every completed
//! render so it can support cursor placement and hit testing.

use crate::grapheme::StyledGraphemes;

/// A widget's position in its width-independent, newline-delimited content.
///
/// `column` is a terminal display-cell offset, not a character index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContentPosition {
    pub row: usize,
    pub column: usize,
}

/// A position after the content has been wrapped for a terminal width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisualPosition {
    pub row: usize,
    pub column: usize,
}

/// A position on the terminal screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScreenPosition {
    pub row: u16,
    pub column: u16,
}

/// A content position associated with a renderer item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidgetPosition<K> {
    pub index: K,
    pub row: usize,
    pub column: usize,
}

impl<K> WidgetPosition<K> {
    pub fn content_position(&self) -> ContentPosition {
        ContentPosition {
            row: self.row,
            column: self.column,
        }
    }
}

/// Horizontal overflow behavior applied by the renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidthMode {
    /// Continue content on subsequent visual rows.
    #[default]
    Wrap,
    /// Keep one visual row per logical row and append an ellipsis when needed.
    Truncate,
}

/// Layout constraints requested by a widget.
///
/// `max_height` is a preference rather than a terminal allocation. The renderer
/// combines it with the laid-out content height, terminal height, and the other
/// non-empty widgets. `width_mode` controls whether each logical row wraps or is
/// truncated with an ellipsis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetLayout {
    pub max_height: Option<usize>,
    pub width_mode: WidthMode,
}

/// Width-independent content and metadata created by a widget.
///
/// `graphemes` contains the widget's complete content, not only its visible
/// window. This lets the renderer preserve a viewport while a cursor moves inside
/// it and scroll only when the cursor crosses an edge.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CreatedGraphemes {
    pub graphemes: StyledGraphemes,
    pub layout: WidgetLayout,
    pub cursor: Option<ContentPosition>,
}

impl From<StyledGraphemes> for CreatedGraphemes {
    fn from(graphemes: StyledGraphemes) -> Self {
        Self {
            graphemes,
            ..Self::default()
        }
    }
}

/// A viewport assigned to one renderer item.
///
/// `content_row` is a row in the terminal-width-dependent visual layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetViewport {
    pub screen_row: u16,
    pub height: u16,
    pub content_row: usize,
}

impl WidgetViewport {
    /// Scrolls the minimum distance needed to include `position`.
    ///
    /// Moving inside the current viewport leaves `content_row` unchanged.
    pub fn scroll_to_include(&mut self, position: VisualPosition) -> ViewportChange {
        if self.height == 0 {
            return ViewportChange::Unchanged;
        }

        let previous = self.content_row;
        let height = self.height as usize;

        if position.row < self.content_row {
            self.content_row = position.row;
        } else if position.row >= self.content_row.saturating_add(height) {
            self.content_row = position.row.saturating_add(1).saturating_sub(height);
        }

        if self.content_row == previous {
            ViewportChange::Unchanged
        } else {
            ViewportChange::Scrolled
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportChange {
    Unchanged,
    Scrolled,
}

/// Projects widget state into complete, width-independent styled content.
///
/// Implementations must not inspect the terminal size or apply viewport
/// clipping. Terminal-dependent work belongs to [`crate::render::Renderer`].
pub trait Widget {
    fn create_graphemes(&self) -> CreatedGraphemes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_does_not_scroll_while_position_is_visible() {
        let mut viewport = WidgetViewport {
            height: 3,
            content_row: 4,
            ..Default::default()
        };

        assert_eq!(
            viewport.scroll_to_include(VisualPosition { row: 6, column: 0 }),
            ViewportChange::Unchanged
        );
        assert_eq!(viewport.content_row, 4);
    }

    #[test]
    fn viewport_scrolls_the_minimum_distance() {
        let mut viewport = WidgetViewport {
            height: 3,
            content_row: 4,
            ..Default::default()
        };

        assert_eq!(
            viewport.scroll_to_include(VisualPosition { row: 7, column: 0 }),
            ViewportChange::Scrolled
        );
        assert_eq!(viewport.content_row, 5);

        assert_eq!(
            viewport.scroll_to_include(VisualPosition { row: 2, column: 0 }),
            ViewportChange::Scrolled
        );
        assert_eq!(viewport.content_row, 2);
    }
}
