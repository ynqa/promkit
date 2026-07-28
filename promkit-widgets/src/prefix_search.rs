use promkit_core::{
    ContentPosition, CreatedGraphemes, Widget, WidgetLayout, grapheme::StyledGraphemes,
};

#[path = "prefix_search/prefix_search.rs"]
mod inner;
pub use inner::PrefixSearch;
pub mod config;
pub use config::Config;

/// State for searching, selecting, and rendering prefix-matched candidates.
#[derive(Clone)]
pub struct State {
    /// Prefix-search data and its current selection.
    pub prefix_search: PrefixSearch,
    /// Rendering configuration.
    pub config: Config,
}

impl Widget for State {
    fn create_graphemes(&self) -> CreatedGraphemes {
        let cursor = StyledGraphemes::from(&self.config.cursor);
        let cursor_width = cursor.widths();
        let lines = self
            .prefix_search
            .candidates()
            .enumerate()
            .map(|(index, candidate)| {
                if Some(index) == self.prefix_search.selected() {
                    let line =
                        StyledGraphemes::from_iter([&cursor, &StyledGraphemes::from(candidate)]);
                    if let Some(style) = self.config.active_item_style {
                        line.apply_style(style)
                    } else {
                        line
                    }
                } else {
                    let line = StyledGraphemes::from_iter([
                        &StyledGraphemes::from(" ".repeat(cursor_width)),
                        &StyledGraphemes::from(candidate),
                    ]);
                    if let Some(style) = self.config.inactive_item_style {
                        line.apply_style(style)
                    } else {
                        line
                    }
                }
            });

        CreatedGraphemes {
            graphemes: StyledGraphemes::from_lines(lines),
            layout: WidgetLayout {
                max_height: self.config.lines,
                ..Default::default()
            },
            cursor: self
                .prefix_search
                .selected()
                .map(|selected| ContentPosition {
                    row: selected,
                    column: 0,
                }),
        }
    }
}

impl State {
    /// Interprets a content position as a prefix-search candidate.
    pub fn hit_at(&self, position: ContentPosition) -> Option<PrefixSearchHit> {
        self.prefix_search
            .candidate_at(position.row)
            .map(|_| PrefixSearchHit::Select {
                index: position.row,
            })
    }
}

/// Semantic targets exposed by the prefix-search widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixSearchHit {
    Select { index: usize },
}

#[cfg(test)]
mod tests {
    use promkit_core::{ContentPosition, Widget};

    use super::{Config, PrefixSearch, PrefixSearchHit, State};

    #[test]
    fn projects_trie_matches_and_resolves_hits() {
        let mut prefix_search: PrefixSearch = ["apple", "applet", "application", "banana"]
            .into_iter()
            .collect();
        prefix_search.search("app");
        let mut state = State {
            prefix_search,
            config: Config {
                lines: Some(3),
                ..Default::default()
            },
        };

        let created = state.create_graphemes();

        assert_eq!(
            created.graphemes.to_string(),
            "❯ apple\n  applet\n  application"
        );
        assert_eq!(created.layout.max_height, Some(3));
        assert_eq!(created.cursor, Some(ContentPosition { row: 0, column: 0 }));
        assert_eq!(
            state.hit_at(ContentPosition { row: 2, column: 80 }),
            Some(PrefixSearchHit::Select { index: 2 })
        );
        assert_eq!(state.hit_at(ContentPosition { row: 3, column: 0 }), None);

        state.prefix_search.move_to(1);
        let created = state.create_graphemes();

        assert_eq!(
            created.graphemes.to_string(),
            "  apple\n❯ applet\n  application"
        );
        assert_eq!(created.cursor, Some(ContentPosition { row: 1, column: 0 }));
    }
}
