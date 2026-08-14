//! Width-independent widget output and the coordinate types used during layout.
//!
//! Widgets project state into styled content. Alongside that content they return
//! a [`WidgetLayout`] hint and, when applicable, a logical [`ContentPosition`]
//! for the cursor. [`crate::render::Renderer`] owns terminal-dependent wrapping,
//! truncation, vertical viewport allocation, and scrolling.
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

/// Vertical sizing behavior applied by the renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Height {
    /// Use the content height, subject to [`WidgetLayout::max_height`].
    #[default]
    Content,
    /// Share the height left after content-sized widgets with other fill widgets,
    /// subject to [`WidgetLayout::max_height`].
    Fill,
}

/// Layout constraints requested by a widget.
///
/// `max_height` is a preference rather than a terminal allocation. The renderer
/// combines it with the laid-out content height, terminal height, and the other
/// non-empty widgets. [`Height::Content`] uses that content-derived height,
/// while [`Height::Fill`] shares the remaining terminal height equally with
/// other fill widgets. `width_mode` controls whether each logical row wraps or
/// is truncated with an ellipsis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetLayout {
    pub height: Height,
    pub max_height: Option<usize>,
    pub width_mode: WidthMode,
}

/// Width-independent content and metadata created by a widget.
///
/// `graphemes` normally contains the widget's complete content. Widgets with
/// large backing stores may return a bounded projection from
/// [`Widget::create_graphemes_in_viewport`].
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

/// Projects widget state into width-independent styled content.
pub trait Widget {
    /// Creates the widget's complete content.
    fn create_graphemes(&self) -> CreatedGraphemes;

    /// Creates content bounded by a terminal viewport.
    ///
    /// The default implementation returns the complete content. Large widgets
    /// can override this method to avoid projecting rows that cannot be shown.
    /// Wrapping and truncation remain the renderer's responsibility.
    fn create_graphemes_in_viewport(&self, _width: u16, _height: u16) -> CreatedGraphemes {
        self.create_graphemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod widget_viewport {
        use super::*;

        mod scroll_to_include {
            use super::*;

            #[test]
            fn does_not_scroll_while_the_position_is_visible() {
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
            fn scrolls_the_minimum_distance_to_include_the_position() {
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
    }
}
