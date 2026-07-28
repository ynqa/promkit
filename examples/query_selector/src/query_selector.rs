use promkit::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
                MouseEvent, MouseEventKind,
            },
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    widgets::{
        listbox::{self, Listbox, ListboxHit},
        text::{self, Text},
        text_editor::{self, TextEditorHit},
    },
    Prompt, Signal, TerminalModes, TerminalSession,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Title,
    Query,
    List,
}

type Filter = fn(&str, &[String]) -> Vec<String>;

struct QuerySelector {
    renderer: Option<SharedRenderer<Index>>,
    title: text::State,
    query: text_editor::State,
    items: Vec<String>,
    list: listbox::State,
    filter: Filter,
}

impl QuerySelector {
    fn new(items: impl IntoIterator<Item = impl ToString>, filter: Filter) -> Self {
        let items: Vec<_> = items.into_iter().map(|item| item.to_string()).collect();
        Self {
            renderer: None,
            title: text::State {
                text: Text::from("What number do you like?"),
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            query: text_editor::State {
                config: text_editor::Config {
                    prefix: "❯❯ ".into(),
                    prefix_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    active_char_style: ContentStyle {
                        background_color: Some(Color::DarkCyan),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            list: listbox::State {
                listbox: Listbox::from(items.iter()),
                config: listbox::Config {
                    cursor: "❯ ".into(),
                    active_item_style: Some(ContentStyle {
                        foreground_color: Some(Color::DarkCyan),
                        ..Default::default()
                    }),
                    inactive_item_style: Some(ContentStyle::default()),
                    lines: Some(5),
                },
            },
            items,
            filter,
        }
    }

    fn handle_event(&mut self, event: &Event) -> anyhow::Result<Signal> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Ok(Signal::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Err(anyhow::anyhow!("ctrl+c")),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.handle_click(*column, *row);
                return Ok(Signal::Continue);
            }
            _ => {}
        }

        let before = self.query.texteditor.text_without_cursor().to_string();
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.list.listbox.backward();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => {
                self.list.listbox.forward();
            }
            Event::Key(key)
                if key.kind == KeyEventKind::Press && key.state == KeyEventState::NONE =>
            {
                Self::edit(&mut self.query, key);
            }
            _ => {}
        }

        let after = self.query.texteditor.text_without_cursor().to_string();
        if before != after {
            self.list.listbox = Listbox::from((self.filter)(&after, &self.items));
        }
        Ok(Signal::Continue)
    }

    fn edit(query: &mut text_editor::State, key: &KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Left, KeyModifiers::NONE) => {
                query.texteditor.backward();
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                query.texteditor.forward();
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => query.texteditor.move_to_head(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => query.texteditor.move_to_tail(),
            (KeyCode::Backspace, KeyModifiers::NONE) => query.texteditor.erase(),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => query.texteditor.erase_all(),
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                match query.config.edit_mode {
                    text_editor::Mode::Insert => query.texteditor.insert(ch),
                    text_editor::Mode::Overwrite => query.texteditor.overwrite(ch),
                }
            }
            _ => {}
        }
    }

    fn handle_click(&mut self, column: u16, row: u16) {
        let Some(position) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
        else {
            return;
        };
        match position.index {
            Index::Query => {
                if let Some(TextEditorHit::Cursor { index }) =
                    self.query.hit_at(position.content_position())
                {
                    self.query.texteditor.move_to(index);
                }
            }
            Index::List => {
                if let Some(ListboxHit::Select { index }) =
                    self.list.hit_at(position.content_position())
                {
                    self.list.listbox.move_to(index);
                }
            }
            Index::Title => {}
        }
    }

    async fn render(&self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Renderer not initialized"))?;
        renderer
            .update([
                (Index::Title, self.title.create_graphemes()),
                (Index::Query, self.query.create_graphemes()),
                (Index::List, self.list.create_graphemes()),
            ])
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for QuerySelector {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::Query, self.query.create_graphemes()),
                    (Index::List, self.list.create_graphemes()),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = self.handle_event(event);
        self.render().await?;
        signal
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.list.listbox.get().to_string())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = {
        let modes =
            TerminalModes::RAW_MODE | TerminalModes::HIDDEN_CURSOR | TerminalModes::MOUSE_CAPTURE;
        let _terminal_session = TerminalSession::try_new(modes)?;
        QuerySelector::new(0..100, |text, items| {
            text.parse::<usize>()
                .map(|query| {
                    items
                        .iter()
                        .filter(|item| query <= item.parse::<usize>().unwrap_or_default())
                        .cloned()
                        .collect()
                })
                .unwrap_or_else(|_| items.to_vec())
        })
        .run()
        .await?
    };
    println!("result: {:?}", ret);
    Ok(())
}
