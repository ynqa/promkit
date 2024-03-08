use std::any::Any;

use crate::{
    core::cursor::CompositeCursor,
    crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent,
        MouseEventKind,
    },
    grapheme::{trim, StyledGraphemes},
    keymap::KeymapManager,
    pane::Pane,
    AsAny, Error, EventAction, Result,
};

use super::{JsonNode, JsonPath, JsonSyntaxKind};

#[derive(Clone)]
pub struct JsonBundle {
    roots: Vec<JsonNode>,
    cursor: CompositeCursor<Vec<JsonSyntaxKind>>,
}

impl JsonBundle {
    pub fn new<I: IntoIterator<Item = JsonNode>>(iter: I) -> Self {
        let roots: Vec<JsonNode> = iter.into_iter().collect();
        Self {
            roots: roots.clone(),
            cursor: CompositeCursor::new(roots.iter().map(|r| r.flatten_visibles())),
        }
    }

    pub fn roots(&self) -> &Vec<JsonNode> {
        &self.roots
    }

    pub fn flatten_kinds(&self) -> Vec<JsonSyntaxKind> {
        self.roots
            .iter()
            .flat_map(|root| root.flatten_visibles().into_iter())
            .collect()
    }

    pub fn current_bundle_path_from_root(&self) -> JsonPath {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();
        let kind = self.cursor.bundle()[index][inner].clone();
        let binding = vec![];
        let path = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayEntry { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapEntry { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => binding,
        };

        path.clone()
    }

    pub fn toggle(&mut self) {
        let (index, inner) = self.cursor.current_bundle_index_and_inner_position();

        let kind = self.cursor.bundle()[index][inner].clone();
        let route = match kind {
            JsonSyntaxKind::ArrayStart { path, .. } => path,
            JsonSyntaxKind::ArrayFolded { path, .. } => path,
            JsonSyntaxKind::MapStart { path, .. } => path,
            JsonSyntaxKind::MapFolded { path, .. } => path,
            _ => return,
        };

        self.roots[index].toggle(&route);
        self.cursor = CompositeCursor::new_with_position(
            self.roots.iter().map(|r| r.flatten_visibles()),
            self.cursor.cross_contents_position(),
        );
    }

    pub fn backward(&mut self) -> bool {
        self.cursor.backward()
    }

    pub fn forward(&mut self) -> bool {
        self.cursor.forward()
    }

    /// Moves the cursor to the head of the JSON tree.
    pub fn move_to_head(&mut self) {
        self.cursor.move_to_head()
    }

    /// Moves the cursor to the tail of the JSON tree.
    pub fn move_to_tail(&mut self) {
        self.cursor.move_to_tail()
    }
}

/// Default key bindings for JSON navigation and manipulation.
///
/// | Key                    | Action
/// | :--------------------- | :-------------------------------------------
/// | <kbd>Enter</kbd>       | Exit the JSON viewer
/// | <kbd>Ctrl + C</kbd>    | Interrupt the current operation
/// | <kbd>↑</kbd>           | Move the cursor up to the previous node
/// | <kbd>↓</kbd>           | Move the cursor down to the next node
/// | <kbd>Space</kbd>       | Toggle fold/unfold on the current node
pub fn default_keymap(renderer: &mut Renderer, event: &Event) -> Result<EventAction> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Ok(EventAction::Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => return Err(Error::Interrupted("ctrl+c".into())),

        // Move cursor.
        Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.bundle.backward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            renderer.bundle.backward();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.bundle.forward();
        }
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: _,
            row: _,
            modifiers: KeyModifiers::NONE,
        }) => {
            renderer.bundle.forward();
        }

        // Fold/Unfold
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.bundle.toggle();
        }

        _ => (),
    }
    Ok(EventAction::Continue)
}

#[derive(Clone)]
pub struct Renderer {
    pub bundle: JsonBundle,
    pub keymap: KeymapManager<Self>,
    pub theme: super::Theme,
}

impl crate::Renderer for Renderer {
    fn make_pane(&self, width: u16) -> Pane {
        let layout = self
            .bundle
            .flatten_kinds()
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                if i == self.bundle.cursor.cross_contents_position() {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::Renderer::indent_level(kind, &self.theme)),
                        ),
                        super::Renderer::gen_syntax_style(kind, &self.theme)
                            .apply_attribute_to_all(self.theme.active_item_attribute),
                    ])
                } else {
                    StyledGraphemes::from_iter([
                        StyledGraphemes::from(
                            " ".repeat(super::Renderer::indent_level(kind, &self.theme)),
                        ),
                        super::Renderer::gen_syntax_style(kind, &self.theme),
                    ])
                    .apply_attribute_to_all(self.theme.inactive_item_attribute)
                }
            })
            .map(|row| trim(width as usize, &row))
            .collect::<Vec<StyledGraphemes>>();

        Pane::new(
            layout,
            self.bundle.cursor.cross_contents_position(),
            self.theme.lines,
        )
    }

    fn handle_event(&mut self, event: &Event) -> Result<EventAction> {
        (self.keymap.get())(self, event)
    }

    fn postrun(&mut self) {
        self.bundle.move_to_head()
    }
}

impl AsAny for Renderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
