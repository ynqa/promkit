use std::path::Path;

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
        structured::tree::{self, Document, TreeHit},
        text::{self, Text},
    },
    Prompt, Signal,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Title,
    Tree,
}

struct TreePrompt {
    renderer: Option<SharedRenderer<Index>>,
    title: text::State,
    tree: tree::State,
}

impl TreePrompt {
    fn new(document: Document) -> Self {
        Self {
            renderer: None,
            title: text::State {
                text: Text::from("Select a directory or file"),
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            tree: tree::State {
                document,
                config: tree::Config {
                    folded_symbol: "▶︎ ".into(),
                    unfolded_symbol: "▼ ".into(),
                    active_item_style: ContentStyle {
                        foreground_color: Some(Color::DarkCyan),
                        ..Default::default()
                    },
                    inactive_item_style: ContentStyle::default(),
                    indent: 2,
                    lines: Some(10),
                    show_line_numbers: true,
                },
            },
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
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.tree.document.up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.tree.document.down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.tree.document.toggle(),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => self.toggle_at(*column, *row),
            _ => {}
        }
        Ok(Signal::Continue)
    }

    fn toggle_at(&mut self, column: u16, row: u16) {
        let Some(position) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
            .filter(|position| position.index == Index::Tree)
        else {
            return;
        };
        if let Some(TreeHit::Toggle { row_index }) = self.tree.hit_at(position.content_position()) {
            self.tree.document.toggle_at(row_index);
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
                (Index::Tree, self.tree.create_graphemes()),
            ])
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for TreePrompt {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::Tree, self.tree.create_graphemes()),
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

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.tree.document.get())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../promkit/src");
    let document = Document::from_path(&root)?;
    let ret = TreePrompt::new(document).run().await?;
    println!("result: {:?}", ret);
    Ok(())
}
