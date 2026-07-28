use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

#[path = "listbox/listbox.rs"]
mod inner;
pub use inner::Listbox;
pub mod config;
pub use config::Config;

/// Represents the state of a `Listbox` component, including its appearance and behavior.
/// This state includes the currently selected item, styles for active and inactive items,
/// and the number of lines available for rendering the listbox.
#[derive(Clone)]
pub struct State {
    /// The `Listbox` component to be rendered.
    pub listbox: Listbox,
    /// Configuration for rendering and behavior.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let lines = self.listbox.items().iter().enumerate().map(|(i, item)| {
            if Some(i) == self.listbox.selected() {
                let init =
                    StyledGraphemes::from_iter([&StyledGraphemes::from(&self.config.cursor), item]);
                if let Some(style) = &self.config.active_item_style {
                    init.apply_style(*style)
                } else {
                    init
                }
            } else {
                let init = StyledGraphemes::from_iter([
                    &StyledGraphemes::from(
                        " ".repeat(StyledGraphemes::from(&self.config.cursor).widths()),
                    ),
                    item,
                ]);
                if let Some(style) = &self.config.inactive_item_style {
                    init.apply_style(*style)
                } else {
                    init
                }
            }
        });

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(lines),
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: self.listbox.selected().map(|selected| ContentPosition {
                row: selected,
                column: 0,
            }),
        }
    }
}

impl State {
    /// Interprets a listbox content position as a selectable item.
    ///
    /// Wrapped visual rows are normalized to their logical content row by the
    /// core renderer before this method resolves the item index.
    pub fn hit_at(&self, position: ContentPosition) -> Option<ListboxHit> {
        self.listbox
            .items()
            .get(position.row)
            .map(|_| ListboxHit::Select {
                index: position.row,
            })
    }
}

/// Semantic targets exposed by the listbox widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListboxHit {
    Select { index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod state {
        use super::*;

        mod hit_at {
            use super::*;

            #[test]
            fn resolves_item_rows() {
                let state = State {
                    listbox: Listbox::from(["first", "second"]),
                    config: Config::default(),
                };

                assert_eq!(
                    state.hit_at(ContentPosition { row: 1, column: 20 }),
                    Some(ListboxHit::Select { index: 1 })
                );
                assert_eq!(state.hit_at(ContentPosition { row: 2, column: 0 }), None);
            }
        }
    }
}
